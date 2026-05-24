use bevy::prelude::*;

pub mod emitter;
pub mod extended_material;
pub mod material;
pub mod spawner;
pub mod state;

pub use emitter::{despawn_expired, spawn_mesh_from_emitter, spawn_single_mesh};
pub use material::{CustomMaterial, insert_sample_material, reload_shaders};
pub use spawner::refresh_sample_mesh;
pub use state::{SampleModel, SampleState};

use my_meshes::FlatRing3d;

use crate::sample::{emitter::spawn_single_gltf_scene, material::replace_material_of_scene};

pub struct SamplePlugin;

impl Plugin for SamplePlugin {
    fn build(&self, app: &mut App) {
        // #[cfg(target_family = "wasm")]
        // bevy::tasks::block_on(async {
        //     app.world_mut()
        //         .resource_mut::<GltfExtensionHandlers>()
        //         .0
        //         .write()
        //         .await
        //         .push(Box::new(ReplaceMaterialGltfExtensionHandler))
        // });
        // #[cfg(not(target_family = "wasm"))]
        // app.world_mut()
        //     .resource_mut::<GltfExtensionHandlers>()
        //     .0
        //     .write_blocking()
        //     .push(Box::new(ReplaceMaterialGltfExtensionHandler));

        app.add_plugins((
            MaterialPlugin::<CustomMaterial>::default(),
            extended_material::MyExtendedMaterialPlugin::default(),
        ))
        .insert_resource(SampleState::default())
        .add_systems(
            Update,
            refresh_sample_mesh.run_if(resource_changed::<SampleState>),
        )
        .add_systems(Update, replace_material_of_scene)
        .add_systems(Update, insert_sample_material)
        .add_systems(
            Update,
            (
                despawn_expired,
                spawn_single_mesh,
                spawn_mesh_from_emitter::<Cuboid, FlatRing3d>,
                spawn_single_gltf_scene,
            )
                .chain(),
        );
    }
}
