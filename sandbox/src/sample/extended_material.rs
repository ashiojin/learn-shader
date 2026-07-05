
use bevy::{
    asset::uuid::Uuid, pbr::{ExtendedMaterial, MaterialExtension}, prelude::*, render::render_resource::AsBindGroup, shader::ShaderRef,
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

impl MaterialExtension for MyExtension {
    fn fragment_shader() -> bevy::shader::ShaderRef {
        ShaderRef::Handle(EXTENDED_MATERIAL_WGSL_UUID.into())
    }

    fn alpha_mode() -> Option<AlphaMode> {
        Some(AlphaMode::Blend)
    }

    fn vertex_shader() -> bevy::shader::ShaderRef {
        ShaderRef::Handle(EXTENDED_MATERIAL_WGSL_UUID.into())
    }

    fn specialize(
        _pipeline: &bevy::pbr::MaterialExtensionPipeline,
        descriptor: &mut bevy::material::descriptor::RenderPipelineDescriptor,
        layout: &bevy::mesh::MeshVertexBufferLayoutRef,
        _key: bevy::pbr::MaterialExtensionKey<Self>,
    ) -> std::prelude::v1::Result<(), bevy::material::specialize::SpecializedMeshPipelineError>
    {
        assert!(descriptor.vertex.buffers.len() == 1, "Expected only one vertex buffer layout for the mesh");

        // Mesh attributes for Bevy PBR
        // constructed in `specialize()` of `SpecializedMeshPipeline` for `MeshPipeline`
        // https://github.com/bevyengine/bevy/blob/c6f634ca9f406d68ba5109d921247b654cb42c10/crates/bevy_pbr/src/render/mesh.rs#L3284
        let base_mesh_attirbutes_list = [
            // in specialize()
            (Mesh::ATTRIBUTE_POSITION, 0),
            (Mesh::ATTRIBUTE_NORMAL, 1),
            (Mesh::ATTRIBUTE_UV_0, 2),
            (Mesh::ATTRIBUTE_UV_1, 3),
            (Mesh::ATTRIBUTE_TANGENT, 4),
            (Mesh::ATTRIBUTE_COLOR, 5),
            // in setup_morph_and_skinning_defs()
            //   https://github.com/bevyengine/bevy/blob/0eac08ae5da33f39d64ad148740c34c14b38c481/crates/bevy_pbr/src/render/mesh.rs#L3275
            (Mesh::ATTRIBUTE_JOINT_INDEX, 6),
            (Mesh::ATTRIBUTE_JOINT_WEIGHT, 7),
        ];

        let mut vertex_attriutes = Vec::new();
        // reconstruct the vertex attributes list for the mesh.
        for (attr, loc) in base_mesh_attirbutes_list {
            if layout.0.contains(attr) {
                vertex_attriutes.push(attr.at_shader_location(loc));
            }
        }

        if layout.0.contains(my_meshes::ATTRIBUTE_TIME) {
            vertex_attriutes.push(my_meshes::ATTRIBUTE_TIME.at_shader_location(10));
            descriptor.vertex.shader_defs.push("MY_MESHES_ATTRIBUTE_TIME".into());
            if let Some(shader_defs) = descriptor.fragment.as_mut().map(|f| &mut f.shader_defs) {
                shader_defs.push("MY_MESHES_ATTRIBUTE_TIME".into());
            }
            info!("Found my_meshes::ATTRIBUTE_TIME in mesh layout, adding shader def MY_MESHES_ATTRIBUTE_TIME");
        } else {
            info!("my_meshes::ATTRIBUTE_TIME not found in mesh layout, shader def MY_MESHES_ATTRIBUTE_TIME not added");
        }

        let vertex_layout = layout.0.get_layout(&vertex_attriutes)?;
        descriptor.vertex.buffers = vec![vertex_layout];

        Ok(())
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
