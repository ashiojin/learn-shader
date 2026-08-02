use bevy::prelude::*;
use my_meshes::{SplineTrail, SplineTrailPoint};
use rand::distr::Distribution;

use super::state::SampleModel;
use crate::{
    random::RandomSource,
    sample::scene_mod::{TrailEmitter, TrailHistory},
};

#[derive(Component, Debug, Clone)]
pub struct SingleMeshEmitter {
    pub mesh: Handle<Mesh>,
}

pub fn spawn_single_mesh(
    mut commands: Commands,
    query: Query<(Entity, &SingleMeshEmitter, &Transform), Added<SingleMeshEmitter>>,
) {
    for (_entity, emitter, transform) in query.iter() {
        commands.spawn((Mesh3d(emitter.mesh.clone()), *transform, SampleModel::Mesh));
    }
}

#[derive(Component, Debug, Clone)]
pub struct RandomPositionEmitter<SS: ShapeSample + Clone + 'static, M: Meshable + 'static> {
    /// The shape sample to use for generating random positions.
    pub shape_sample: SS,

    pub mesh: M,

    pub only_boundary: bool,
    pub mesh_lifetime: MeshLifetimePattern,

    pub spawn_pattern: SpawnPattern,
}

#[derive(Debug, Clone, Copy)]
pub enum MeshLifetimePattern {
    Const(f32),
}

#[derive(Debug, Clone, Copy)]
pub enum SpawnPattern {
    //OnlyAtStart(usize),
    FixedRate {
        rate_per_sec: f32, // particles per second
    },
}

#[allow(clippy::too_many_arguments)]
pub fn spawn_mesh_from_emitter<SS, M>(
    time: Res<Time>,
    mut commands: Commands,
    query: Query<(Entity, &RandomPositionEmitter<SS, M>, &Transform)>,
    mut rnd: ResMut<RandomSource>,

    mut meshes: ResMut<Assets<Mesh>>,
) where
    SS: ShapeSample + 'static + Sync + Send + Clone,
    M: Meshable + 'static + Send + Sync,
    <SS as ShapeSample>::Output: Into<Vec3>,
{
    for (_entity, emitter, emitter_transform) in query.iter() {
        match emitter.spawn_pattern {
            SpawnPattern::FixedRate { rate_per_sec } => {
                let now = time.elapsed_secs();
                let prev = now - time.delta_secs();
                let num_at_prev = (prev * rate_per_sec).floor() as usize;
                let num_at_now = (now * rate_per_sec).floor() as usize;
                let particles_to_spawn = num_at_now.saturating_sub(num_at_prev);

                let positions: Vec<Vec3> = if emitter.only_boundary {
                    let dist = emitter.shape_sample.clone().boundary_dist();
                    dist.sample_iter(&mut rnd.rnd_mut())
                        .take(particles_to_spawn)
                        .map(|pos| pos.into())
                        .collect()
                } else {
                    let dist = emitter.shape_sample.clone().interior_dist();
                    dist.sample_iter(&mut rnd.rnd_mut())
                        .take(particles_to_spawn)
                        .map(|pos| pos.into())
                        .collect()
                };

                for pos in positions {
                    let h_mesh = meshes.add(emitter.mesh.mesh());
                    let transform = *emitter_transform * Transform::from_translation(pos);
                    let mesh_entity = commands
                        .spawn((
                            Mesh3d(h_mesh),
                            transform,
                            MeshLifetime {
                                lifetime: match emitter.mesh_lifetime {
                                    MeshLifetimePattern::Const(l) => l,
                                },
                                spwawned_at: time.elapsed_secs(),
                            },
                            SampleModel::Mesh,
                        ))
                        .id();

                    debug!(
                        "Spawned mesh entity {:?} at position {:?}",
                        mesh_entity, pos
                    );
                }
            }
        }
    }
}

#[derive(Component, Clone, Copy)]
pub struct MeshLifetime {
    lifetime: f32,
    spwawned_at: f32,
}

pub fn despawn_expired(
    time: Res<Time>,
    mut commands: Commands,
    query: Query<(Entity, &MeshLifetime)>,
) {
    for (entity, mesh_lifetime) in query.iter() {
        if time.elapsed_secs() - mesh_lifetime.spwawned_at >= mesh_lifetime.lifetime {
            commands.entity(entity).try_despawn();
            // try_despawn is used to avoid despawning entities that have already been despawned by other systems.
            // ex. Changing models may cause multiple despawn with the same entity in the same frame.
        }
    }
}

#[derive(Component, Debug, Clone)]
pub struct SingleGltfEmitter {
    pub gltf_path: String,
    pub scene_idx: usize,
}

#[derive(Component, Debug, Clone)]
pub struct AutoPlay {
    node_idx: AnimationNodeIndex,
    named_node_idx_list: Vec<(String, AnimationNodeIndex)>,
    repeat: bool,
    graph_handle: AnimationGraphHandle,
    /// The entity of the AnimationPlayer that is playing this animation. This is set when the AnimationPlayer is spawned and the AutoPlay system finds it.
    player_entity: Option<Entity>,
}
impl AutoPlay {
    pub fn new(
        node_idx: AnimationNodeIndex,
        named_node_idx_list: Vec<(String, AnimationNodeIndex)>,
        repeat: bool,
        graph_handle: AnimationGraphHandle,
    ) -> Self {
        Self {
            node_idx,
            named_node_idx_list,
            repeat,
            graph_handle,
            player_entity: None,
        }
    }

    pub fn node_idx(&self) -> AnimationNodeIndex {
        self.node_idx
    }

    pub fn set_node_idx(&mut self, node_idx: AnimationNodeIndex) {
        self.node_idx = node_idx;
    }

    pub fn node_idx_list(&self) -> &Vec<(String, AnimationNodeIndex)> {
        &self.named_node_idx_list
    }

    pub fn repeat(&self) -> bool {
        self.repeat
    }

    pub fn graph_handle(&self) -> AnimationGraphHandle {
        self.graph_handle.clone()
    }

    pub fn player_entity(&self) -> Option<Entity> {
        self.player_entity
    }

    pub fn set_player_entity(&mut self, entity: Entity) {
        self.player_entity = Some(entity);
    }
}

pub fn spawn_single_gltf_scene(
    mut commands: Commands,
    mut query: Query<(Entity, &SingleGltfEmitter, &Transform), Added<SingleGltfEmitter>>,
    asset_server: Res<AssetServer>,
) {
    for (_entity, emitter, transform) in query.iter_mut() {
        debug!(
            "spawn_single_gltf_scene: {:?}, {:?}",
            emitter.gltf_path, emitter.scene_idx
        );
        commands.spawn((
            WorldAssetRoot(asset_server.load(
                GltfAssetLabel::Scene(emitter.scene_idx).from_asset(emitter.gltf_path.clone()),
            )),
            *transform,
            SampleModel::Scene,
            WaitingAnimationGraph(emitter.gltf_path.clone()),
        ));
    }
}

#[derive(Component, Debug, Clone)]
pub struct WaitingAnimationGraph(String);

#[derive(Component, Debug, Clone)]
pub struct WaitingAnimationPlayer;

pub fn insert_animation_graph_with_all_animation(
    mut commands: Commands,
    q_scene: Query<(Entity, &WaitingAnimationGraph), With<WaitingAnimationGraph>>,
    asset_server: Res<AssetServer>,
    mut animation_graphs: ResMut<Assets<AnimationGraph>>,
    gltf: Res<Assets<Gltf>>,
) {
    for (entity, notyet) in q_scene.iter() {
        let Some(gltf) = gltf.get(&asset_server.load(notyet.0.clone())) else {
            error!("Gltf asset not loaded yet: {:?}", notyet.0);
            continue;
        };

        let mut graph = AnimationGraph::new();
        let mut first_node = None;
        let mut node_list = Vec::new();
        for (name, clip) in gltf.named_animations.iter() {
            info!("Adding animation clip to graph: {:?}", entity);
            let node = graph.add_clip(clip.clone(), 0.0, graph.root);
            node_list.push((name.to_string(), node));

            if first_node.is_none() {
                first_node = Some(node);
            }
        }
        if let Some(first_node) = first_node {
            // set w=1.0
            let node = graph.get_mut(first_node).unwrap();
            node.weight = 1.0;

            // === DEBUG ===
            // let mut str = String::new();
            // graph.save(&mut str).unwrap();
            // info!("AnimationGraph for entity {:?}:{}", entity, str);
            // === DEBUG ===

            let h_graph = animation_graphs.add(graph);
            commands.entity(entity).try_insert((
                AutoPlay::new(first_node, node_list, true, AnimationGraphHandle(h_graph)),
                WaitingAnimationPlayer,
            ));
        }
        commands
            .entity(entity)
            .try_remove::<WaitingAnimationGraph>();
    }
}

#[allow(clippy::type_complexity)]
pub fn auto_play(
    mut commands: Commands,
    mut q_animation_players: Query<(Entity, &mut AnimationPlayer), With<AnimationPlayer>>,
    mut q_auto_play: Query<(Entity, &mut AutoPlay), With<WaitingAnimationPlayer>>,
    q_children: Query<&ChildOf>,
) {
    for (player_entity, mut player) in q_animation_players.iter_mut() {
        debug!(
            "Checking AutoPlay for entity {:?} with AnimationPlayer",
            player_entity
        );
        if let Some(auto_play_entity) = q_children
            .iter_ancestors(player_entity)
            .find_map(|ancestor| q_auto_play.get(ancestor).map(|(e, _)| e).ok())
        {
            let mut auto_play = q_auto_play.get_mut(auto_play_entity).unwrap().1;
            info!(
                "Found AutoPlay for entity {:?}, node: {:?}, looped: {:?}",
                player_entity,
                auto_play.node_idx(),
                auto_play.repeat()
            );

            auto_play.set_player_entity(player_entity);

            commands
                .entity(player_entity)
                .try_insert(auto_play.graph_handle().clone());

            commands
                .entity(auto_play_entity)
                .try_remove::<WaitingAnimationPlayer>();

            player.play(auto_play.node_idx());
        }
    }
}

pub fn auto_play_next(
    mut q_animation_players: Query<(Entity, &mut AnimationPlayer), With<AnimationPlayer>>,
    mut q_auto_play: Query<(Entity, &mut AutoPlay)>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
) {
    for (root_entity, mut auto_play) in q_auto_play.iter_mut() {
        let Some(player_entity) = auto_play.player_entity() else {
            continue;
        };
        let Some((_, mut player)) = q_animation_players.get_mut(player_entity).ok() else {
            continue;
        };
        let Some(mut graph) = graphs.get_mut(&auto_play.graph_handle().0) else {
            continue;
        };

        // if the current animation is finished, set the next one with weight 1.0 & play it
        if player
            .animation(auto_play.node_idx())
            .map(|a| a.is_finished())
            .unwrap_or(false)
        {
            // all node weight reset to 0.0
            for (_, node_idx) in auto_play.node_idx_list() {
                if let Some(node) = graph.get_mut(*node_idx) {
                    node.weight = 0.0;
                }
            }
            player.stop_all();

            let current_node = auto_play.node_idx();
            let next_node = auto_play
                .node_idx_list()
                .iter()
                .map(|(_name, idx)| idx)
                .cycle()
                .skip_while(|&&n| n != current_node)
                .nth(1)
                .copied()
                .unwrap_or(current_node);

            debug!(
                "AutoPlay: Switching from node {:?} to node {:?} for entity {:?}",
                current_node, next_node, root_entity
            );
            if let Some(node) = graph.get_mut(next_node) {
                node.weight = 1.0;
            }
            player.play(next_node);
            auto_play.set_node_idx(next_node);
        }
    }
}

#[allow(clippy::type_complexity)]
#[allow(clippy::too_many_arguments)]
pub fn spawn_trail_from_emitter(
    mut commands: Commands,
    mut q_trail_emitter: Query<(
        Entity,
        &TrailEmitter,
        &Mesh3d,
        &GlobalTransform,
        Option<&mut TrailHistory>,
    )>,
    mut assets_meshes: ResMut<Assets<Mesh>>,
    q_mesh_3d: Query<&Mesh3d>,
    q_animation_players: Query<(Entity, &AnimationPlayer)>,
    q_children: Query<&ChildOf>,
    q_auto_play: Query<(Entity, &AutoPlay)>,
    trail_config: Option<Res<crate::config::TrailConfig>>,
    time: Res<Time>,
) {
    let current_time = time.elapsed_secs();

    for (entity, trail_emitter, mesh, global_transform, opt_history) in q_trail_emitter.iter_mut() {
        let Some(mesh_asset) = assets_meshes.get(&mesh.0) else {
            continue;
        };

        // Here we assume there are two vertices
        let Some(vertices) = mesh_asset
            .attribute(Mesh::ATTRIBUTE_POSITION)
            .and_then(|attr| attr.as_float3())
        else {
            continue;
        };
        if vertices.is_empty() {
            continue;
        }

        // For now we use only 2 vertices, first and last one, to determine the trail positions.
        let vertices = [vertices[0], vertices[vertices.len() - 1]];
        // Get global positions by transforming local vertices using the global transform
        let global_positions: Vec<Vec3> = vertices
            .iter()
            .map(|v| global_transform.transform_point(Vec3::from(*v)))
            .collect();

        let current_root = global_positions[0];
        let current_tip = global_positions[1];

        let mut spawn_trail = false;

        // Find AutoPlay from ancestors of the Entity
        let opt_auto_play = q_children
            .iter_ancestors(entity)
            .find_map(|e| q_auto_play.get(e).map(|(_, auto_play)| auto_play).ok());

        if let Some(auto_play) = opt_auto_play
            && let Some(player_entity) = auto_play.player_entity()
            && let Ok((_entity, player)) = q_animation_players.get(player_entity)
        {
            for (playing_anim_nidx, playing_animation) in player.playing_animations() {
                let seek_time = playing_animation.seek_time();
                if trail_emitter.timings_of(*playing_anim_nidx).iter().any(|timing| timing.is_on_time(seek_time)) {
                    debug!(
                        "TrailEmitter for entity {:?} is active at elapsed time {:?}, spawning trail",
                        entity, playing_animation.seek_time()
                    );
                    spawn_trail = true;
                } else {
                    debug!(
                        "TrailEmitter for entity {:?} has no timing for playing animation node {:?}, skipping trail spawn",
                        entity, playing_anim_nidx
                    );
                }
            }
        } else {
            debug!(
                "AnimationPlayer for entity {:?} does not exist or is not playing, skipping trail spawn",
                entity
            );
        }

        // Handle the history queue
        let mut history = match opt_history {
            Some(h) => h,
            None => {
                let initial_mode = trail_config
                    .as_ref()
                    .map(|c| c.mode)
                    .unwrap_or(trail_emitter.mode());
                let mut h = TrailHistory::new(initial_mode, trail_emitter.subdivisions());
                if spawn_trail {
                    h.points.push_back(SplineTrailPoint {
                        root: current_root,
                        tip: current_tip,
                        time: current_time,
                        break_before: false,
                    });
                    h.was_active = true;
                }
                commands.entity(entity).insert(h);
                continue;
            }
        };

        // Detect the start of a new burst: the emitter is active now but was
        // idle last frame. The first point pushed for this burst is flagged so
        // the mesh builder cuts the ribbon instead of bridging the idle gap.
        if spawn_trail && !history.was_active && !history.points.is_empty() {
            history.pending_break = true;
        }

        // Push new point if spawning is active
        if spawn_trail {
            // Avoid inserting identical positions at the exact same timestamp to prevent zero-length divisions
            let should_push = if let Some(last) = history.points.back() {
                last.root.distance_squared(current_root) > 1e-6
                    || last.tip.distance_squared(current_tip) > 1e-6
                    || (current_time - last.time) > 0.05
            } else {
                true
            };

            if should_push {
                let break_before = history.pending_break;
                history.points.push_back(SplineTrailPoint {
                    root: current_root,
                    tip: current_tip,
                    time: current_time,
                    break_before,
                });
                history.pending_break = false;
            }
        }

        // Remember this frame's emission state for next-frame gap detection.
        history.was_active = spawn_trail;

        // Prune old points
        let cutoff_time = current_time - trail_emitter.lifetime();
        while history.points.len() > 1 && history.points[0].time < cutoff_time {
            history.points.pop_front();
        }

        // Rebuild or clean up trail entity.
        //
        // `build_mesh` yields `None` when the history holds no drawable ribbon --
        // e.g. a break has just isolated the single point left over from the
        // previous burst. Such a frame must be treated exactly like an empty
        // history: pushing a zero-vertex mesh into `Assets<Mesh>` makes bevy's
        // MeshAllocator free the old allocation, skip re-allocating, and still
        // attempt the vertex copy, logging
        // "Use-after-free: attempted to copy element data for an unallocated key".
        let new_mesh = if history.points.len() >= 2 {
            SplineTrail::new(
                history.points.iter().cloned().collect(),
                history.subdivisions,
            )
            .build_mesh(history.mode)
        } else {
            None
        };

        if let Some(new_mesh) = new_mesh {
            if let Some(trail_ent) = history.trail_entity {
                if let Ok(mesh_3d) = q_mesh_3d.get(trail_ent)
                    && let Some(mut mesh_asset) = assets_meshes.get_mut(&mesh_3d.0)
                {
                    *mesh_asset = new_mesh;
                }
            } else {
                let mesh_handle = assets_meshes.add(new_mesh);
                let spawned_ent = commands
                    .spawn((
                        Mesh3d(mesh_handle),
                        Transform::IDENTITY,
                        GlobalTransform::IDENTITY,
                        SampleModel::Mesh,
                    ))
                    .id();
                history.trail_entity = Some(spawned_ent);
            }
        } else {
            // Nothing drawable: no segments left (or faded completely)
            if let Some(trail_ent) = history.trail_entity {
                commands.entity(trail_ent).try_despawn();
                history.trail_entity = None;
            }
        }
    }
}
