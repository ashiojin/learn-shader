use bevy::prelude::*;
use rand::distr::Distribution;

use crate::random::RandomSource;
use super::state::SampleMesh;

#[derive(Component, Debug, Clone)]
pub struct SingleMeshEmitter {
    pub mesh: Handle<Mesh>,
}

pub fn spawn_single_mesh(
    mut commands: Commands,
    query: Query<(Entity, &SingleMeshEmitter, &Transform), Added<SingleMeshEmitter>>,
) {
    for (_entity, emitter, transform) in query.iter() {
        commands.spawn((
            Mesh3d(emitter.mesh.clone()),
            *transform,
            SampleMesh,
        ));
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
    FixedRate{
        rate_per_sec : f32, // particles per second
    },
}

#[allow(clippy::too_many_arguments)]
pub fn spawn_mesh_from_emitter<SS, M>(
    time: Res<Time>,
    mut commands: Commands,
    query: Query<(Entity, &RandomPositionEmitter<SS, M>, &Transform)>,
    mut rnd: ResMut<RandomSource>,

    mut meshes: ResMut<Assets<Mesh>>,
)
where
    SS: ShapeSample + 'static + Sync + Send + Clone,
    M: Meshable + 'static + Send + Sync,
    <SS as ShapeSample>::Output: Into<Vec3>,
{
    for (_entity, emitter, emitter_transform) in query.iter() {
        match emitter.spawn_pattern {
            SpawnPattern::FixedRate { rate_per_sec, } => {
                let now = time.elapsed_secs();
                let prev = now - time.delta_secs();
                let num_at_prev = (prev * rate_per_sec).floor() as usize;
                let num_at_now = (now * rate_per_sec).floor() as usize;
                let particles_to_spawn = num_at_now.saturating_sub(num_at_prev);

                let positions: Vec<Vec3> = if emitter.only_boundary {
                    let dist = emitter.shape_sample.clone().boundary_dist();
                    dist.sample_iter(&mut rnd.rnd_mut()).take(particles_to_spawn).map(|pos| pos.into()).collect()
                } else {
                    let dist = emitter.shape_sample.clone().interior_dist();
                    dist.sample_iter(&mut rnd.rnd_mut()).take(particles_to_spawn).map(|pos| pos.into()).collect()
                };

                for pos in positions {
                    let h_mesh = meshes.add(emitter.mesh.mesh());
                    let transform = *emitter_transform * Transform::from_translation(pos);
                    let mesh_entity = commands.spawn((
                        Mesh3d(h_mesh),
                        transform,
                        MeshLifetime {
                            lifetime: match emitter.mesh_lifetime {
                                MeshLifetimePattern::Const(l) => l,
                            },
                            spwawned_at: time.elapsed_secs(),
                        },
                        SampleMesh,
                    )).id();

                    debug!("Spawned mesh entity {:?} at position {:?}", mesh_entity, pos);
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
            commands.entity(entity).despawn();
        }
    }
}
