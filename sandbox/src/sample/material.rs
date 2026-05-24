use bevy::gltf::GltfMaterialExtras;
use bevy::{
    color::palettes::css, pbr::ExtendedMaterial, prelude::*, render::render_resource::AsBindGroup,
};

use super::state::SampleModel;
use super::state::{SampleMaterialType, SampleState};
use crate::sample::extended_material::{MY_EXTENSION_SHADER_PATH, MyExtendedMaterial, MyExtension};

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct CustomMaterial {
    #[uniform(129)]
    spawned_at: f32,
}

pub const SHADER_ASSET_PATH: &str = "shaders/fragment.wgsl";
impl Material for CustomMaterial {
    fn fragment_shader() -> bevy::shader::ShaderRef {
        SHADER_ASSET_PATH.into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Blend
    }
}

#[allow(clippy::too_many_arguments)]
pub fn insert_sample_material(
    mut commands: Commands,
    mut custom_materials: ResMut<Assets<CustomMaterial>>,
    mut extended_materials: ResMut<Assets<MyExtendedMaterial>>,
    mut standard_materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
    sample_state: Res<SampleState>,
    q_sample_models: Query<(Entity, &SampleModel), Added<SampleModel>>,
    time: Res<Time>,
) {
    for (entity, model) in q_sample_models.iter() {
        if *model != SampleModel::Mesh {
            continue;
        }
        // add material
        match sample_state.material_type {
            SampleMaterialType::User => {
                commands
                    .entity(entity)
                    .try_insert(MeshMaterial3d(custom_materials.add(CustomMaterial {
                        spawned_at: time.elapsed_secs(),
                    })));
            }
            SampleMaterialType::UserExtended => {
                let base_material = StandardMaterial {
                    base_color: css::LIGHT_GRAY.into(),
                    alpha_mode: AlphaMode::Blend, // TODO:
                    ..Default::default()
                };
                let material = ExtendedMaterial {
                    base: base_material,
                    extension: MyExtension::new(
                        LinearRgba::new(1.0, 0.0, 0.0, 0.25),
                        time.elapsed_secs(),
                    ),
                };
                commands
                    .entity(entity)
                    .try_insert(MeshMaterial3d(extended_materials.add(material)));
            }
            SampleMaterialType::UvTest1024 => {
                let texture_handle = asset_server.load(myshaderlib::path_to_uv_test1024());
                let material = StandardMaterial {
                    base_color_texture: Some(texture_handle),
                    ..Default::default()
                };
                commands
                    .entity(entity)
                    .try_insert(MeshMaterial3d(standard_materials.add(material)));
            }
        }
    }
}

// pub fn replace_material_of_scene(
//     scene_ready: On<SceneInstanceReady>,
//     mut commands: Commands,
//     mut custom_materials: ResMut<Assets<CustomMaterial>>,
//     mut extended_materials: ResMut<Assets<MyExtendedMaterial>>,
//     mut standard_materials: ResMut<Assets<StandardMaterial>>,
//     sample_state: Res<SampleState>,
//     asset_server: Res<AssetServer>,
//     children: Query<&Children>,
//     time: Res<Time>,
// ) {
// }

const REPLACE_TARGET_PROP_NAME: &str = "Ashiojin_bevy_material_type";
const REPLACE_TARGET_PROP_VALUE: &str = "sandbox_target";
#[allow(clippy::too_many_arguments)]
pub fn replace_material_of_scene(
    q_material: Query<(Entity, &MeshMaterial3d<StandardMaterial>, &GltfMaterialExtras), Added<GltfMaterialExtras>>,
    mut commands: Commands,
    mut custom_materials: ResMut<Assets<CustomMaterial>>,
    mut extended_materials: ResMut<Assets<MyExtendedMaterial>>,
    mut standard_materials: ResMut<Assets<StandardMaterial>>,
    sample_state: Res<SampleState>,
    asset_server: Res<AssetServer>,
    time: Res<Time>,
) {
    for (entity, standard_material, material_extras) in q_material.iter() {
        let extras_json = material_extras.value.clone();
        let untyped_extras: serde_json::Value = serde_json::from_str(&extras_json).expect("Failed to parse material extras as JSON");
        debug!(
            "Material Entity {:?} : extras as json: {:?}",
            entity, untyped_extras
        );

        if let serde_json::Value::Object(ref map) = untyped_extras
            && map.iter().any(|(k, v)| {
                k == REPLACE_TARGET_PROP_NAME
                    && v.as_str()
                        .map(|s| s == REPLACE_TARGET_PROP_VALUE)
                        .unwrap_or(false)
            })
        {
            // 1. remove MeshMaterial3d<StandardMaterial>
            commands
                .entity(entity)
                .remove::<MeshMaterial3d<StandardMaterial>>();
            // 2. add a custom material according to the sample state
            match sample_state.material_type {
                SampleMaterialType::User => {
                    let custom_material = CustomMaterial {
                        spawned_at: time.elapsed_secs(),
                    };
                    let asset_handle = custom_materials.add(custom_material);
                    commands.entity(entity).insert(MeshMaterial3d(asset_handle));
                }
                SampleMaterialType::UserExtended => {
                    let base_material = standard_materials.get(standard_material.id()).cloned().unwrap_or_default();
                    let material = ExtendedMaterial {
                        base: base_material,
                        extension: MyExtension::new(
                            LinearRgba::new(1.0, 0.0, 0.0, 0.25),
                            time.elapsed_secs(),
                        ),
                    };
                    let asset_handle = extended_materials.add(material);
                    commands.entity(entity).insert(MeshMaterial3d(asset_handle));
                }
                SampleMaterialType::UvTest1024 => {
                    let texture_handle = asset_server.load(myshaderlib::path_to_uv_test1024());
                    let material = StandardMaterial {
                        base_color_texture: Some(texture_handle),
                        ..Default::default()
                    };
                    let asset_handle = standard_materials.add(material);
                    commands.entity(entity).insert(MeshMaterial3d(asset_handle));
                }
            }
        }
    }
}

// #[derive(Default, Clone)]
// pub struct ReplaceMaterialGltfExtensionHandler;
//
// impl GltfExtensionHandler for ReplaceMaterialGltfExtensionHandler {
//     fn dyn_clone(&self) -> Box<dyn GltfExtensionHandler> {
//         Box::new(self.clone())
//     }
//     fn on_spawn_mesh_and_material(
//         &mut self,
//         _load_context: &mut bevy::asset::LoadContext<'_>,
//         _primitive: &gltf::Primitive,
//         _mesh: &gltf::Mesh,
//         material: &gltf::Material,
//         entity: &mut EntityWorldMut,
//     ) {
//         info!("Material {:?} : extensions: {:?}", material.name(), material.extensions());
//
//         if material
//             .extension_value(REPLACE_TARGET_PROP_NAME)
//             .iter().flat_map(|v| v.as_str())
//             .any(|s| s == REPLACE_TARGET_PROP_VALUE)
//         {
//             // 1. remove MeshMaterial3d<StandardMaterial>
//             entity.remove::<MeshMaterial3d<StandardMaterial>>();
//             // 2. add a custom material according to the sample state
//             let material_type = entity.resource::<SampleState>().material_type;
//             match material_type {
//                 SampleMaterialType::User => {
//                     let custom_material = CustomMaterial {
//                         spawned_at: entity.resource::<Time>().elapsed_secs(),
//                     };
//                     let asset_handle = entity
//                         .resource_mut::<Assets<CustomMaterial>>()
//                         .add(custom_material);
//                     entity.insert(MeshMaterial3d(
//                         asset_handle,
//                     ));
//                 }
//                 SampleMaterialType::UserExtended => {
//                     let base_material = StandardMaterial {
//                         base_color: css::LIGHT_GRAY.into(),
//                         alpha_mode: AlphaMode::Blend, // TODO:
//                         ..Default::default()
//                     };
//                     let material = ExtendedMaterial {
//                         base: base_material,
//                         extension: MyExtension::new(
//                             LinearRgba::new(1.0, 0.0, 0.0, 0.25),
//                             entity.resource::<Time>().elapsed_secs(),
//                         ),
//                     };
//                     let asset_handle = entity
//                         .resource_mut::<Assets<MyExtendedMaterial>>()
//                         .add(material);
//                     entity.insert(MeshMaterial3d(
//                         asset_handle,
//                     ));
//                 }
//                 SampleMaterialType::UvTest1024 => {
//                     let texture_handle = entity
//                         .resource::<AssetServer>()
//                         .load(myshaderlib::path_to_uv_test1024());
//                     let material = StandardMaterial {
//                         base_color_texture: Some(texture_handle),
//                         ..Default::default()
//                     };
//                     let asset_handle = entity
//                         .resource_mut::<Assets<StandardMaterial>>()
//                         .add(material);
//                     entity.insert(MeshMaterial3d(
//                         asset_handle,
//                     ));
//                 }
//             }
//         }
//     }
// }

pub fn reload_shaders(asset_server: &AssetServer) {
    asset_server.reload(SHADER_ASSET_PATH);
    asset_server.reload(MY_EXTENSION_SHADER_PATH);
}
