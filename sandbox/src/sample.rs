use bevy::prelude::*;

mod billboard;
pub mod emitter;
pub mod extended_material;
pub mod material;
pub mod preload;
pub mod scene_mod;
pub mod spawner;
pub mod state;

pub use emitter::{despawn_expired, spawn_mesh_from_emitter, spawn_single_mesh};
pub use material::{
    CustomMaterial, apply_sandbox_materials, insert_sample_material,
};
pub use preload::{AppState, PreloadPlugin};
pub use spawner::refresh_sample_mesh;
pub use state::{SampleModel, SampleState};

use my_meshes::FlatRing3d;

use crate::sample::{
    billboard::add_billboard_component, emitter::{auto_play, spawn_single_gltf_scene, spawn_trail_from_emitter}, extended_material::{
        ReloadReq, load_global_res, request_load_extended_material,
    }, material::{apply_sandbox_fx_meshes, init_custom_material}, scene_mod::draw_gizmo_for_trail_meshes,
};

pub struct SamplePlugin;

impl Plugin for SamplePlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<material::SandboxExtension>();
        app.register_type::<material::SandboxMeshFxConfigExtension>();

        app.insert_resource(material::CustomMaterialShader(
            crate::api_shared::read_wgsl("CustomMaterial").unwrap_or_default(),
        ));
        app.insert_resource(extended_material::ExtendedMaterialShader(
            crate::api_shared::read_wgsl("ExtendedMaterial").unwrap_or_default(),
        ));

        if let Some(mut handlers) = app.world_mut()
            .resource_mut::<bevy::gltf::extensions::GltfExtensionHandlers>()
            .0
            .try_write()
        {
            handlers.push(Box::new(
                crate::sample::material::ReplaceMaterialGltfExtensionHandler,
            ));
        } else {
            warn!("Failed to acquire write lock for GltfExtensionHandlers");
        }

        app.add_plugins((
            MaterialPlugin::<CustomMaterial>::default(),
            extended_material::MyExtendedMaterialPlugin::default(),
        ))
        .add_message::<ReloadReq>()
        .insert_resource(SampleState::default())
        .add_systems(
            Startup,
            (
                request_load_extended_material,
                material::request_load_custom_material,
            ),
        )
        .add_systems(Update, (load_global_res, material::load_custom_material))
        .add_systems(
            Update,
            refresh_sample_mesh
                .run_if(resource_changed::<SampleState>)
                .run_if(in_state(AppState::Running)),
        )
        .add_systems(OnEnter(AppState::Running), trigger_initial_sample)
        .add_systems(
            Update,
            (
                insert_sample_material,
                apply_sandbox_materials,
                apply_sandbox_fx_meshes,
                add_billboard_component,
                draw_gizmo_for_trail_meshes,
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
            Update,
            (
                auto_play,
            ),
        )
        .add_systems(
            PostUpdate,
            spawn_trail_from_emitter.after(TransformSystems::Propagate),
        );

        init_custom_material(app);
    }
}

/// When preloading finishes and the app enters `Running`, force
/// `refresh_sample_mesh` to run once so the default sample is spawned. This
/// avoids relying on first-frame change-detection timing across the state
/// transition.
fn trigger_initial_sample(mut sample_state: ResMut<SampleState>) {
    sample_state.set_changed();
}
