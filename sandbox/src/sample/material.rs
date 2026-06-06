use std::sync::Mutex;

use bevy::{
    asset::uuid::Uuid,
    color::palettes::css, pbr::ExtendedMaterial, prelude::*, render::render_resource::AsBindGroup,
};

use super::state::SampleModel;
use super::state::{SampleMaterialType, SampleState};
use crate::sample::extended_material::{
    MY_EXTENSION_SHADER_PATH, MyExtendedMaterial, MyExtension, ReloadReq,
};

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct CustomMaterial {
    #[uniform(129)]
    spawned_at: f32,
}

pub const SHADER_ASSET_PATH: &str = "shaders/fragment.wgsl";

static CUSTOM_MATERIAL_WGSL: Mutex<Vec<u8>> = Mutex::new(Vec::new());
const CUSTOM_MATERIAL_WGSL_UUID: Uuid = Uuid::from_u128(0xffff0000aaaabdef1234567890abcdee);
const CUSTOM_MATERIAL_WGSL_PATH: &str = "globals:custom_material.wgsl";

pub fn init_custom_material(mut req_sender: MessageWriter<ReloadReq>) {
    let mut wgsl = CUSTOM_MATERIAL_WGSL.lock().expect("Failed to lock CUSTOM_WGSL");
    wgsl.clear();
    let bytes = include_bytes!("../shaders/fragment.wgsl");
    wgsl.extend_from_slice(bytes);
    req_sender.write(ReloadReq);
}

pub fn load_custom_material(
    mut shaders: ResMut<Assets<Shader>>,
    mut reload_reqs: MessageReader<ReloadReq>,
) {
    let is_requested = reload_reqs.read().any(|_| true);
    if !is_requested {
        return;
    }
    let wgsl = CUSTOM_MATERIAL_WGSL.lock().expect("Failed to lock CUSTOM_WGSL");
    let copy = wgsl.clone();
    let text = String::from_utf8(copy).unwrap();
    let shader = Shader::from_wgsl(text, CUSTOM_MATERIAL_WGSL_PATH.to_string());
    shaders
        .insert(CUSTOM_MATERIAL_WGSL_UUID, shader)
        .expect("Failed to insert shader");
    info!("Reloaded custom material shader");
}

pub fn write_custom_material(wgsl_text: &str) {
    let mut wgsl = CUSTOM_MATERIAL_WGSL.lock().expect("Failed to lock CUSTOM_WGSL");
    wgsl.clear();
    wgsl.extend_from_slice(wgsl_text.as_bytes());
}

pub fn read_custom_material() -> String {
    let wgsl = CUSTOM_MATERIAL_WGSL.lock().expect("Failed to lock CUSTOM_WGSL");
    String::from_utf8(wgsl.clone()).unwrap()
}

impl Material for CustomMaterial {
    fn fragment_shader() -> bevy::shader::ShaderRef {
        bevy::shader::ShaderRef::Handle(CUSTOM_MATERIAL_WGSL_UUID.into())
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

#[derive(Component, Reflect, Default, serde::Deserialize, Debug, Clone)]
#[reflect(Component)]
pub struct SandboxExtension {
    pub shader_type: String,
    pub param1: [f32; 4],
}

const EXTENSION_NAME: &str = "ASHIOJIN_material_sandbox";

#[derive(Default, Clone)]
pub struct ReplaceMaterialGltfExtensionHandler;

impl bevy::gltf::extensions::GltfExtensionHandler for ReplaceMaterialGltfExtensionHandler {
    fn dyn_clone(&self) -> Box<dyn bevy::gltf::extensions::GltfExtensionHandler> {
        Box::new(self.clone())
    }
    fn on_spawn_mesh_and_material(
        &mut self,
        _load_context: &mut bevy::asset::LoadContext<'_>,
        _primitive: &gltf::Primitive,
        _mesh: &gltf::Mesh,
        material: &gltf::Material,
        entity: &mut EntityWorldMut,
    ) {
        if let Some(extension_value) = material.extension_value(EXTENSION_NAME) {
            let sandbox_extension: SandboxExtension =
                serde_json::from_value(extension_value.clone())
                    .expect("Failed to parse ASHIOJIN_material_sandbox extension");

            if sandbox_extension.shader_type == "ASHIOJIN_SANDBOX" {
                entity.insert(sandbox_extension);

                let t = entity.get_resource::<Assets<StandardMaterial>>().is_some();
                info!("{t:?}");
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub fn apply_sandbox_materials(
    mut commands: Commands,
    mut custom_materials: ResMut<Assets<CustomMaterial>>,
    mut extended_materials: ResMut<Assets<MyExtendedMaterial>>,
    mut standard_materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
    sample_state: Res<SampleState>,
    time: Res<Time>,
    #[allow(clippy::type_complexity)] query: Query<
        (
            Entity,
            &SandboxExtension,
            Option<&MeshMaterial3d<StandardMaterial>>,
        ),
        Added<SandboxExtension>,
    >,
) {
    for (entity, sandbox_extension, standard_material_handle) in query.iter() {
        let standard_material = standard_material_handle
            .and_then(|handle| standard_materials.get(&handle.0))
            .cloned();

        // 2. add a custom material according to the sample state
        let material_type = sample_state.material_type;
        match material_type {
            SampleMaterialType::User => {
                let custom_material = CustomMaterial {
                    spawned_at: time.elapsed_secs(),
                };
                let asset_handle = custom_materials.add(custom_material);
                commands.entity(entity).insert(MeshMaterial3d(asset_handle));
            }
            SampleMaterialType::UserExtended => {
                let base_material = standard_material.unwrap_or(StandardMaterial {
                    base_color: css::LIGHT_GRAY.into(),
                    alpha_mode: AlphaMode::Blend,
                    ..Default::default()
                });
                let material = ExtendedMaterial {
                    base: base_material,
                    extension: MyExtension::new(
                        LinearRgba::from_f32_array(sandbox_extension.param1),
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

pub fn reload_shaders(asset_server: &AssetServer) {
    asset_server.reload(SHADER_ASSET_PATH);
    asset_server.reload(MY_EXTENSION_SHADER_PATH);
}
