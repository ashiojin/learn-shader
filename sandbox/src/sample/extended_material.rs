use bevy::{
    pbr::{ExtendedMaterial, MaterialExtension},
    prelude::*,
    render::render_resource::AsBindGroup,
};

#[derive(Asset, AsBindGroup, Reflect, Debug, Clone, Default)]
pub struct MyExtension {
    #[uniform(128)]
    pub param1: LinearRgba,

    #[uniform(129)]
    pub spawned_at: f32,
}
impl MyExtension {
    pub fn new(param1: LinearRgba, spawned_at: f32) -> Self {
        Self {
            param1,
            spawned_at,
        }
    }
}

pub const MY_EXTENSION_SHADER_PATH: &str = "shaders/extended_material.wgsl";
impl MaterialExtension for MyExtension {
    fn fragment_shader() -> bevy::shader::ShaderRef {
        MY_EXTENSION_SHADER_PATH.into()
    }

    fn alpha_mode() -> Option<AlphaMode> {
        Some(AlphaMode::Blend)
    }

    fn vertex_shader() -> bevy::shader::ShaderRef {
        MY_EXTENSION_SHADER_PATH.into()
    }

}

pub type MyExtendedMaterial = ExtendedMaterial<StandardMaterial, MyExtension>;
pub type MyExtendedMaterialPlugin = MaterialPlugin<MyExtendedMaterial>;
