use crate::sample::{
    emitter::{
        MeshLifetimePattern, RandomPositionEmitter, SingleGltfEmitter, SingleMeshEmitter,
        SpawnPattern,
    },
    scene_mod::{AnimationType, AutoAnimation},
};
use bevy::prelude::*;
use my_meshes::{self as meshes, Belt};
use std::f32::consts::PI;

pub use super::state::SampleModel;
use super::state::{SampleEmitter, SampleState, SampleType};

pub fn spawn_sample(sample_type: &SampleType, commands: &mut Commands, meshes: &mut Assets<Mesh>) {
    match sample_type {
        SampleType::Saru => {
            commands.spawn((
                SingleGltfEmitter {
                    gltf_path: "models/saru.glb".to_string(),
                    scene_idx: 0,
                },
                Transform::from_xyz(0., 0., 0.),
                SampleEmitter,
            ));
        }
        SampleType::ArmAndRod => {
            commands.spawn((
                SingleGltfEmitter {
                    gltf_path: "models/poc_arm_and_rod_ex.glb".to_string(),
                    scene_idx: 0,
                },
                Transform::from_xyz(0., 0., 0.).with_scale(Vec3::new(0.3, 0.3, 0.3)),
                SampleEmitter,
                AutoAnimation::new(1, AnimationType::Repeat),
            ));
        }
        SampleType::Plane => {
            commands.spawn((
                SingleMeshEmitter {
                    mesh: meshes.add(Plane3d::new(Vec3::Z, Vec2::new(1.0, 1.0))),
                },
                Transform::from_xyz(0., 0., 0.),
                SampleEmitter,
            ));
        }
        SampleType::Cube => {
            commands.spawn((
                SingleMeshEmitter {
                    mesh: meshes.add(Cuboid::from_length(1.0).mesh()),
                },
                Transform::from_xyz(0., 0., 0.),
                SampleEmitter,
            ));
        }
        SampleType::Cone => {
            commands.spawn((
                SingleMeshEmitter {
                    mesh: meshes.add(Cone::new(0.5, 1.0).mesh()),
                },
                Transform::from_xyz(0., 0., 0.),
                SampleEmitter,
            ));
        }
        SampleType::Sphere => {
            commands.spawn((
                SingleMeshEmitter {
                    mesh: meshes.add(Sphere::new(0.5).mesh()),
                },
                Transform::from_xyz(0., 0., 0.),
                SampleEmitter,
            ));
        }
        SampleType::Ring => {
            commands.spawn((
                SingleMeshEmitter {
                    mesh: meshes.add(
                        meshes::FlatRing3d::new(Dir3::Z, 1.0, 0.25)
                            .with_resolution(32)
                            .mesh(),
                    ),
                },
                Transform::from_xyz(0., 0., 0.),
                SampleEmitter,
            ));
        }
        SampleType::SphericalZone => {
            commands.spawn((
                SingleMeshEmitter {
                    mesh: meshes.add(
                        meshes::SphericalZone::new(0.5, 7. * PI / 16.0, 9. * PI / 16.0)
                            .with_circle_resolution(64)
                            .with_angle_resolution(8)
                            .with_double_sided(true)
                            .mesh(),
                    ),
                },
                Transform::from_xyz(0., 0., 0.),
                SampleEmitter,
            ));
        }
        SampleType::Belt => {
            let start_point_pos = Vec3::new(-1.0, 0.0, 0.0);
            let end_point_pos = Vec3::new(0.0, 1.0, 0.0);
            let start_point_dir = Dir3::new(start_point_pos).unwrap();
            let end_point_dir = Dir3::new(end_point_pos).unwrap();
            commands.spawn((
                SingleMeshEmitter {
                    mesh: meshes.add(
                        Belt::new(
                            start_point_pos,
                            start_point_dir,
                            end_point_pos,
                            end_point_dir,
                            1.00,
                        )
                        .with_resolution(64)
                        .mesh(),
                    ),
                },
                Transform::from_xyz(0., 0., 0.),
                SampleEmitter,
            ));
        }
        SampleType::Emitter1 => {
            commands.spawn((
                RandomPositionEmitter {
                    spawn_pattern: SpawnPattern::FixedRate { rate_per_sec: 10.0 },
                    shape_sample: Cuboid::from_length(1.0),
                    only_boundary: false,
                    mesh: meshes::FlatRing3d::new(Dir3::Z, 0.1, 0.05).with_resolution(16),
                    mesh_lifetime: MeshLifetimePattern::Const(2.0),
                },
                Transform::from_xyz(0., 0., 0.),
                SampleEmitter,
            ));
        }
    }
}

pub fn refresh_sample_mesh(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    sample_state: Res<SampleState>,
    q_sample_models: Query<Entity, With<SampleModel>>,
    q_sample_emitters: Query<Entity, With<SampleEmitter>>,
) {
    // 1. despawn old sample
    for entity in q_sample_models.iter() {
        commands.entity(entity).try_despawn();
    }
    for entity in q_sample_emitters.iter() {
        commands.entity(entity).try_despawn();
    }

    // 2. spawn new sample
    spawn_sample(&sample_state.sample_type, &mut commands, &mut meshes);
}
