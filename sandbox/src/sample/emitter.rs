use bevy::prelude::*;
use my_meshes::{SplineTrail, SplineTrailPoint};
use rand::distr::Distribution;

use super::state::SampleModel;
use crate::{
    random::RandomSource,
    sample::scene_mod::{
        AutoAnimation, TrailEmitter, TrailHistory,
    },
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
    repeat: bool,
    graph_handle: AnimationGraphHandle,
    /// The entity of the AnimationPlayer that is playing this animation. This is set when the AnimationPlayer is spawned and the AutoPlay system finds it.
    player_entity: Option<Entity>,
}
impl AutoPlay {
    pub fn new(
        node_idx: AnimationNodeIndex,
        repeat: bool,
        graph_handle: AnimationGraphHandle,
    ) -> Self {
        Self {
            node_idx,
            repeat,
            graph_handle,
            player_entity: None,
        }
    }

    pub fn node_idx(&self) -> AnimationNodeIndex {
        self.node_idx
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
    mut query: Query<
        (
            Entity,
            &SingleGltfEmitter,
            &Transform,
            Option<&AutoAnimation>,
        ),
        Added<SingleGltfEmitter>,
    >,
    asset_server: Res<AssetServer>,
    mut animation_graphs: ResMut<Assets<AnimationGraph>>,
) {
    for (_entity, emitter, transform, o_anime) in query.iter_mut() {
        debug!(
            "spawn_single_gltf_scene: {:?}, {:?}, {:?}",
            emitter.gltf_path, emitter.scene_idx, o_anime
        );
        let cmd = &mut commands.spawn((
            WorldAssetRoot(asset_server.load(
                GltfAssetLabel::Scene(emitter.scene_idx).from_asset(emitter.gltf_path.clone()),
            )),
            *transform,
            SampleModel::Scene,
        ));

        if let Some(anime) = o_anime {
            // Add animation graph
            let mut animation_grpah = AnimationGraph::new();
            let h_clip = asset_server.load(
                GltfAssetLabel::Animation(anime.clip_index()).from_asset(emitter.gltf_path.clone()),
            );
            let node = animation_grpah.add_clip(h_clip, 1.0, animation_grpah.root);
            let h_graph = animation_graphs.add(animation_grpah);
            //let cmd = cmd.insert(AnimationGraphHandle(h_graph));

            match anime.animation_type() {
                crate::sample::scene_mod::AnimationType::Repeat => {
                    cmd.try_insert(AutoPlay::new(node, true, AnimationGraphHandle(h_graph)));
                }
            }

            debug!(
                "Added animation graph: {:?}, node: {:?}, clip_index: {:?}, animation_type: {:?}",
                cmd.id(),
                node,
                anime.clip_index(),
                anime.animation_type()
            );
        }
    }
}

pub fn auto_play(
    mut commands: Commands,
    mut q_animation_players: Query<(Entity, &mut AnimationPlayer), Added<AnimationPlayer>>,
    mut q_auto_play: Query<(Entity, &mut AutoPlay)>,
    q_children: Query<&ChildOf>,
) {
    for (entity, mut player) in q_animation_players.iter_mut() {
        debug!(
            "Checking AutoPlay for entity {:?} with AnimationPlayer",
            entity
        );
        if let Some(auto_play_entity) = q_children
            .iter_ancestors(entity)
            .find_map(|ancestor| q_auto_play.get(ancestor).map(|(e, _)| e).ok())
        {
            let mut auto_play = q_auto_play.get_mut(auto_play_entity).unwrap().1;
            info!(
                "Found AutoPlay for entity {:?}, node: {:?}, looped: {:?}",
                entity,
                auto_play.node_idx(),
                auto_play.repeat()
            );

            auto_play.set_player_entity(entity);

            commands
                .entity(entity)
                .try_insert(auto_play.graph_handle().clone());

            if auto_play.repeat() {
                player.play(auto_play.node_idx()).repeat();
            } else {
                player.play(auto_play.node_idx());
            }
        }
    }
}

pub fn spawn_trail_from_emitter(
    mut commands: Commands,
    mut q_trail_emitter: Query<
        (
            Entity,
            &TrailEmitter,
            &Mesh3d,
            &GlobalTransform,
            Option<&mut TrailHistory>,
        ),
    >,
    mut assets_meshes: ResMut<Assets<Mesh>>,
    q_mesh_3d: Query<&Mesh3d>,
    q_animation_players: Query<(Entity, &AnimationPlayer)>,
    q_children: Query<&ChildOf>,
    q_auto_play: Query<(Entity, &AutoPlay)>,
    trail_config: Option<Res<crate::config::TrailConfig>>,
    time: Res<Time>,
) {
    let current_time = time.elapsed_secs();

    for (entity, trail_emitter, mesh, global_transform, opt_history) in
        q_trail_emitter.iter_mut()
    {
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

        let mut spawn_trail = true;

        // Find AutoPlay from ancestors of the Entity
        let opt_auto_play = q_children
            .iter_ancestors(entity)
            .find_map(|e| q_auto_play.get(e).map(|(_, auto_play)| auto_play).ok());

        if let Some(auto_play) = opt_auto_play {
            if let Some(timing) = trail_emitter.timing()
                && let Some(player_entity) = auto_play.player_entity()
                && let Ok((_entity, player)) = q_animation_players.get(player_entity)
            {
                if let Some(seek_time) = player
                    .animation(auto_play.node_idx())
                    .map(|a| a.seek_time())
                {
                    if !timing.is_active(seek_time) {
                        debug!(
                            "TrailEmitter for entity {:?} is not active at elapsed time {:?}, skipping trail spawn",
                            entity, seek_time
                        );
                        spawn_trail = false;
                    }
                } else {
                    debug!(
                        "AnimationPlayer for entity {:?} does not have animation node {:?} yet, skipping trail spawn",
                        player_entity,
                        auto_play.node_idx()
                    );
                    spawn_trail = false;
                }
            }
        } else {
            info!("Not found AutoPlay: e: {:?}", entity);
            spawn_trail = false;
        }

        // Handle the history queue
        let mut history = match opt_history {
            Some(h) => h,
            None => {
                let initial_mode = trail_config.as_ref().map(|c| c.mode).unwrap_or(trail_emitter.mode());
                let mut h = TrailHistory::new(initial_mode, trail_emitter.subdivisions());
                if spawn_trail {
                    h.points.push_back(SplineTrailPoint {
                        root: current_root,
                        tip: current_tip,
                        time: current_time,
                    });
                }
                commands.entity(entity).insert(h);
                continue;
            }
        };

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
                history.points.push_back(SplineTrailPoint {
                    root: current_root,
                    tip: current_tip,
                    time: current_time,
                });
            }
        }

        // Prune old points
        let cutoff_time = current_time - trail_emitter.lifetime();
        while history.points.len() > 1 && history.points[0].time < cutoff_time {
            history.points.pop_front();
        }

        // Rebuild or clean up trail entity
        if history.points.len() >= 2 {
            let spline_trail = SplineTrail::new(
                history.points.iter().cloned().collect(),
                history.subdivisions,
            );
            let new_mesh = spline_trail.build_mesh(history.mode);

            if let Some(trail_ent) = history.trail_entity {
                if let Ok(mesh_3d) = q_mesh_3d.get(trail_ent) {
                    if let Some(mut mesh_asset) = assets_meshes.get_mut(&mesh_3d.0) {
                        *mesh_asset = new_mesh;
                    }
                }
            } else {
                let mesh_handle = assets_meshes.add(new_mesh);
                let spawned_ent = commands.spawn((
                    Mesh3d(mesh_handle),
                    Transform::IDENTITY,
                    GlobalTransform::IDENTITY,
                    SampleModel::Mesh,
                )).id();
                history.trail_entity = Some(spawned_ent);
            }
        } else {
            // No segments left (or faded completely)
            if let Some(trail_ent) = history.trail_entity {
                commands.entity(trail_ent).try_despawn();
                history.trail_entity = None;
            }
        }
    }
}
