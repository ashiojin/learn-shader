use std::thread::current;

use bevy::{color::palettes::css, prelude::*};
use my_meshes::Belt;
use rand::distr::Distribution;

use super::state::SampleModel;
use crate::{random::RandomSource, sample::scene_mod::{AutoAnimation, CurrentTrailPositions, PreviousTrailPositions, TrailEmitter}};

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
pub struct AutoPlay(AnimationNodeIndex, bool, AnimationGraphHandle);

pub fn spawn_single_gltf_scene(
    mut commands: Commands,
    mut query: Query<(Entity, &SingleGltfEmitter, &Transform, Option<&AutoAnimation>), Added<SingleGltfEmitter>>,
    asset_server: Res<AssetServer>,
    mut animation_graphs: ResMut<Assets<AnimationGraph>>,
) {
    for (_entity, emitter, transform, o_anime) in query.iter_mut() {
        debug!("spawn_single_gltf_scene: {:?}, {:?}, {:?}", emitter.gltf_path, emitter.scene_idx, o_anime);
        let cmd = &mut commands.spawn((
            SceneRoot(asset_server.load(
                GltfAssetLabel::Scene(emitter.scene_idx).from_asset(
                emitter.gltf_path.clone()),
            )),
            *transform,
            SampleModel::Scene,
        ));

        if let Some(anime) = o_anime {
            // Add animation graph
            let mut animation_grpah = AnimationGraph::new();
            let h_clip = asset_server.load(GltfAssetLabel::Animation(anime.clip_index()).from_asset(emitter.gltf_path.clone()));
            let node = animation_grpah.add_clip(h_clip, 1.0, animation_grpah.root);
            let h_graph = animation_graphs.add(animation_grpah);
            //let cmd = cmd.insert(AnimationGraphHandle(h_graph));

            match anime.animation_type() {
                crate::sample::scene_mod::AnimationType::Repeat => {
                    cmd.insert(AutoPlay(node, true, AnimationGraphHandle(h_graph)));
                }
            }

            debug!("Added animation graph: {:?}, node: {:?}, clip_index: {:?}, animation_type: {:?}", cmd.id(), node, anime.clip_index(), anime.animation_type());
        }
    }
}

pub fn auto_play(
    mut commands: Commands,
    mut q_animation_players: Query<(Entity, &mut AnimationPlayer), Added<AnimationPlayer>>,
    q_auto_play: Query<(Entity, &AutoPlay)>,
    q_children: Query<&ChildOf>,
) {
    for (entity, mut player) in q_animation_players.iter_mut() {
        debug!("Checking AutoPlay for entity {:?} with AnimationPlayer", entity);
         if let Some((_e, auto_play)) = q_children.iter_ancestors(entity).find_map(|ancestor| {
            q_auto_play.get(ancestor).ok()
         }) {
            debug!("Found AutoPlay for entity {:?}, node: {:?}, looped: {:?}", entity, auto_play.0, auto_play.1);
            commands.entity(entity).insert(auto_play.2.clone());

            if auto_play.1 {
                player.play(auto_play.0).repeat();
            } else {
                player.play(auto_play.0);
            }
         }
    }
}

pub fn spawn_trail_from_emmiter(
    mut commands: Commands,
    query: Query<(Entity, &TrailEmitter, &CurrentTrailPositions, &PreviousTrailPositions)>,
    mut meshes: ResMut<Assets<Mesh>>,
    time: Res<Time>,
) {
    // Make a `Belt` between the current and previous positions of the emitter, and spawn a mesh for it. The mesh should have a lifetime equal to the trail lifetime of the emitter.
    for (_entity, trail_emitter, current_pos, previous_pos) in query.iter() {
        let org = current_pos.begin(); // for transform of the mesh entity
        let v_c = Vec3::ZERO; // current_pos.begin() - current_pos.begin() = Vec3::ZERO, we will use the `org` as the origin of the mesh, so the current position will be at (0, 0, 0) in the local space of the mesh
        let d_c = (current_pos.end() - current_pos.begin()).normalize();
        let v_p = previous_pos.begin() - org;
        let d_p = (previous_pos.end() - previous_pos.begin()).normalize();
        let w = (current_pos.begin() - current_pos.end()).length();

        commands.spawn((
            Mesh3d(meshes.add(
                Belt::new(v_p, Dir3::new(d_p).unwrap(), v_c, Dir3::new(d_c).unwrap(), w)
                // TODO: We want to specify additionally the width of the belt at the start and end
                // positions separately, to allow for tapering the trail. For now we will just use the same width for both ends.
                // We also need to add timestamps to the vertices to allow fading out the trail
                // over time in the shader. So we will need another meshable shape that allows us to specify custom vertex attributes.
                    .with_resolution(8)
                    .mesh(),
            )),
            Transform::from_translation(org),
            MeshLifetime {
                lifetime: trail_emitter.trail_lifetime(),
                spwawned_at: time.elapsed_secs(),
            },
            SampleModel::Mesh,
        ));
    }
}
