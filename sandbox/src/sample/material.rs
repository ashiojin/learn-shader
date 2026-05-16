use bevy::{
    color::palettes::css, pbr::ExtendedMaterial, prelude::*, render::render_resource::AsBindGroup,
};

use super::state::SampleMesh;
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
    mut materials: ResMut<Assets<CustomMaterial>>,
    mut extended_materials: ResMut<Assets<MyExtendedMaterial>>,
    mut standard_materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
    sample_state: Res<SampleState>,
    q_sample_meshes: Query<Entity, Added<SampleMesh>>,
    time: Res<Time>,
) {
    for entity in q_sample_meshes.iter() {
        // add material
        match sample_state.material_type {
            SampleMaterialType::User => {
                commands
                    .entity(entity)
                    .try_insert(MeshMaterial3d(materials.add(CustomMaterial {
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

pub fn reload_shaders(asset_server: &AssetServer) {
    asset_server.reload(SHADER_ASSET_PATH);
    asset_server.reload(MY_EXTENSION_SHADER_PATH);
}
