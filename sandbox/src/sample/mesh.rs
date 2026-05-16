use crate::sample::emitter::{
    MeshLifetimePattern, RandomPositionEmitter, SampleEmitter, SpawnPattern,
};
use bevy::prelude::*;
use my_meshes::{self as meshes, Belt};
use std::f32::consts::PI;

use super::state::SampleState;

#[derive(Component, Debug, Clone)]
pub struct SampleMesh;

#[derive(Debug, Clone, Default)]
pub enum SampleType {
    #[default]
    Plane,
    Cube,
    Cone,
    Sphere,
    Ring,
    SphericalZone,
    Belt,
    Emitter1,
}

impl SampleType {
    pub fn get_next(&self) -> Self {
        match self {
            SampleType::Plane => SampleType::Cube,
            SampleType::Cube => SampleType::Cone,
            SampleType::Cone => SampleType::Sphere,
            SampleType::Sphere => SampleType::Ring,
            SampleType::Ring => SampleType::SphericalZone,
            SampleType::SphericalZone => SampleType::Belt,
            SampleType::Belt => SampleType::Emitter1,
            SampleType::Emitter1 => SampleType::Plane,
        }
    }

    pub fn spawn(&self, commands: &mut Commands, meshes: &mut Assets<Mesh>) {
        let entity = match self {
            SampleType::Plane => commands
                .spawn(Mesh3d(
                    meshes.add(Plane3d::new(Vec3::Z, Vec2::new(1.0, 1.0))),
                ))
                .id(),
            SampleType::Cube => commands
                .spawn(Mesh3d(meshes.add(Cuboid::from_length(1.0).mesh())))
                .id(),
            SampleType::Cone => commands
                .spawn(Mesh3d(meshes.add(Cone::new(0.5, 1.0).mesh())))
                .id(),
            SampleType::Sphere => commands
                .spawn(Mesh3d(meshes.add(Sphere::new(0.5).mesh())))
                .id(),
            SampleType::Ring => commands
                .spawn(Mesh3d(
                    meshes.add(
                        meshes::FlatRing3d::new(Dir3::Z, 1.0, 0.25)
                            .with_resolution(32)
                            .mesh(),
                    ),
                ))
                .id(),
            SampleType::SphericalZone => commands
                .spawn(Mesh3d(
                    meshes.add(
                        meshes::SphericalZone::new(0.5, 7. * PI / 16.0, 9. * PI / 16.0)
                            .with_circle_resolution(64)
                            .with_angle_resolution(8)
                            .with_double_sided(true)
                            .mesh(),
                    ),
                ))
                .id(),
            SampleType::Belt => {
                let start_point_pos = Vec3::new(-1.0, 0.0, 0.0);
                let end_point_pos = Vec3::new(0.0, 1.0, 0.0);
                let start_point_dir = Dir3::new(start_point_pos).unwrap();
                let end_point_dir = Dir3::new(end_point_pos).unwrap();
                commands
                    .spawn(Mesh3d(
                        meshes.add(
                            Belt::new(
                                start_point_pos,
                                start_point_dir,
                                end_point_pos,
                                end_point_dir,
                                0.25,
                            )
                            .with_resolution(64)
                            .mesh(),
                        ),
                    ))
                    .id()
            }
            SampleType::Emitter1 => commands
                .spawn(RandomPositionEmitter {
                    spawn_pattern: SpawnPattern::FixedRate { rate_per_sec: 10.0 },
                    shape_sample: Cuboid::from_length(1.0),
                    only_boundary: false,
                    mesh: meshes::FlatRing3d::new(Dir3::Z, 0.1, 0.05).with_resolution(16),
                    mesh_lifetime: MeshLifetimePattern::Const(2.0),
                })
                .id(),
        };
        commands
            .entity(entity)
            .insert((Transform::from_xyz(0., 0., 0.), SampleMesh));
    }
}

pub fn refresh_sample_mesh(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    sample_state: Res<SampleState>,
    q_sample_meshes: Query<Entity, With<SampleMesh>>,
    q_sample_emitters: Query<Entity, (With<SampleEmitter>, Without<SampleMesh>)>,
) {
    // 1. despawn old sample
    for entity in q_sample_meshes.iter() {
        commands.entity(entity).despawn();
    }
    for entity in q_sample_emitters.iter() {
        commands.entity(entity).despawn();
    }

    // 2. spawn new sample
    sample_state.spawn(&mut commands, &mut meshes);
}
