use bevy::prelude::*;

mod billboard;
pub mod emitter;
pub mod extended_material;
pub mod material;
pub mod spawner;
pub mod state;

pub use emitter::{despawn_expired, spawn_mesh_from_emitter, spawn_single_mesh};
pub use material::{
    CustomMaterial, apply_sandbox_materials, insert_sample_material, reload_shaders,
};
pub use spawner::refresh_sample_mesh;
pub use state::{SampleModel, SampleState};

use my_meshes::FlatRing3d;

use crate::sample::{billboard::add_billboard_component, emitter::spawn_single_gltf_scene, extended_material::{ReloadReq, init_global_res, load_global_res}};

pub struct SamplePlugin;

impl Plugin for SamplePlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<material::SandboxExtension>();
        app.world_mut()
            .resource_mut::<bevy::gltf::extensions::GltfExtensionHandlers>()
            .0
            .write_blocking()
            .push(Box::new(
                crate::sample::material::ReplaceMaterialGltfExtensionHandler,
            ));
        app.add_plugins((
            MaterialPlugin::<CustomMaterial>::default(),
            extended_material::MyExtendedMaterialPlugin::default(),
        ))
        .add_message::<ReloadReq>()
        .insert_resource(SampleState::default())
        .add_systems(Startup, init_global_res)
        .add_systems(Update, load_global_res)
        .add_systems(
            Update,
            refresh_sample_mesh.run_if(resource_changed::<SampleState>),
        )
        .add_systems(
            Update,
            (
                insert_sample_material,
                apply_sandbox_materials,
                add_billboard_component,
            ),
        )
        .add_systems(
            Update,
            (
                despawn_expired,
                spawn_single_mesh,
                spawn_mesh_from_emitter::<Cuboid, FlatRing3d>,
                spawn_single_gltf_scene,
            )
                .chain(),
        )
        .add_systems(
            PostUpdate,
            (billboard::update_billboard_transform,)
                .chain()
                .before(TransformSystems::Propagate),
        );
    }
}
