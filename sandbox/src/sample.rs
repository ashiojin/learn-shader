use bevy::prelude::*;

pub mod state;
pub mod spawner;
pub mod material;
pub mod extended_material;
pub mod emitter;

pub use state::{SampleMesh, SampleState};
pub use spawner::refresh_sample_mesh;
pub use material::{CustomMaterial, insert_sample_material, reload_shaders};
pub use emitter::{despawn_expired, spawn_mesh_from_emitter, spawn_single_mesh};

use my_meshes::FlatRing3d;

pub struct SamplePlugin;

impl Plugin for SamplePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            MaterialPlugin::<CustomMaterial>::default(),
            extended_material::MyExtendedMaterialPlugin::default(),
        ))
        .insert_resource(SampleState::default())
        .add_systems(
            Update,
            refresh_sample_mesh.run_if(resource_changed::<SampleState>),
        )
        .add_systems(Update, insert_sample_material)
        .add_systems(
            Update,
            (
                despawn_expired,
                spawn_single_mesh,
                spawn_mesh_from_emitter::<Cuboid, FlatRing3d>,
            )
                .chain(),
        );
    }
}
