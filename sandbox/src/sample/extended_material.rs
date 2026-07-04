
use bevy::{
    asset::uuid::Uuid,
    pbr::{ExtendedMaterial, MaterialExtension},
    prelude::*,
    render::render_resource::AsBindGroup,
    shader::ShaderRef,
};

#[derive(Asset, AsBindGroup, Reflect, Debug, Clone, Default)]
pub struct MyExtension {
    #[uniform(128)]
    pub param1: LinearRgba,

    #[uniform(129)]
    pub spawned_at: f32,
    // NOTE: For now, we use `webgl2` feature flag (default) of bevy, it requires uniform buffer size to be 16 bytes aligned, so we add some padding here.
    #[uniform(129)]
    _weggl2_padding_8b: u32,
    #[uniform(129)]
    _weggl2_padding_12b: u32,
    #[uniform(129)]
    _weggl2_padding_16b: u32,
}
impl MyExtension {
    pub fn new(param1: LinearRgba, spawned_at: f32) -> Self {
        Self {
            param1,
            spawned_at,
            ..default()
        }
    }
}

pub const MY_EXTENSION_SHADER_PATH: &str = "shaders/extended_material.wgsl";
impl MaterialExtension for MyExtension {
    fn fragment_shader() -> bevy::shader::ShaderRef {
        //MY_EXTENSION_SHADER_PATH.into()
        ShaderRef::Handle(EXTENDED_MATERIAL_WGSL_UUID.into())
    }

    fn alpha_mode() -> Option<AlphaMode> {
        Some(AlphaMode::Blend)
    }

    fn vertex_shader() -> bevy::shader::ShaderRef {
        //MY_EXTENSION_SHADER_PATH.into()
        ShaderRef::Handle(EXTENDED_MATERIAL_WGSL_UUID.into())
    }
}

pub type MyExtendedMaterial = ExtendedMaterial<StandardMaterial, MyExtension>;
pub type MyExtendedMaterialPlugin = MaterialPlugin<MyExtendedMaterial>;

#[derive(Resource)]
pub struct ExtendedMaterialShader(pub String);

const EXTENDED_MATERIAL_WGSL_UUID: Uuid = Uuid::from_u128(0xffff0000aaaabdef1234567890abcdef);
const EXTENDED_MATERIAL_WGSL_PATH: &str = "globals:extended_material.wgsl";

pub fn request_load_extended_material(mut req_sender: MessageWriter<ReloadReq>) {
    req_sender.write(ReloadReq);
}

#[derive(Message)]
pub struct ReloadReq;

pub fn load_global_res(
    mut shaders: ResMut<Assets<Shader>>,
    mut reload_reqs: MessageReader<ReloadReq>,
    extended_shader: Res<ExtendedMaterialShader>,
) {
    let is_requested = reload_reqs.read().any(|_| true);
    if !is_requested {
        return;
    }
    let shader = Shader::from_wgsl(extended_shader.0.clone(), EXTENDED_MATERIAL_WGSL_PATH.to_string());
    shaders
        .insert(EXTENDED_MATERIAL_WGSL_UUID, shader)
        .expect("Failed to insert shader");

    info!("Reloaded extended material shader");
}
