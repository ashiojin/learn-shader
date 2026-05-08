use std::f32::consts::PI;

use bevy::{
    mesh::{MeshVertexAttribute, VertexFormat},
    prelude::*,
    render::render_resource::AsBindGroup,
};

const SHADER_ASSET_PATH: &str = "shaders/custom_vertex_attribute.wgsl";

fn main() {
    let asset_root_path = std::env::var("ASSETS_DIR").unwrap_or("assets".into());
    App::new()
        .add_plugins((
            DefaultPlugins.set(AssetPlugin {
                file_path: asset_root_path,
                //watch_for_changes_override: Some(true),
                ..Default::default()
            }),
            MaterialPlugin::<CustomMaterial>::default(),
        ))
        .add_systems(Startup, setup)
        .run();
}

const ATTRIBUTE_BLEND_COLOR: MeshVertexAttribute =
    MeshVertexAttribute::new("BlendColor", 988540917, VertexFormat::Float32x4);

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<CustomMaterial>>,
) {
    let mesh = Mesh::from(
        my_meshes::SphericalZone::new(1.0, 2. * PI / 8., 5. * PI / 8.)
            .with_double_sided(true)
            .with_angle_resolution(32)
            .with_circle_resolution(32)
            .mesh(),
    );
    let vs_num = mesh.count_vertices();
    let blend_colors = Vec::from_iter((0..vs_num).map(|i| {
        let t = i as f32 / vs_num as f32;
        [1.0, t, 0.0, 1.0]
    }));
    let mesh = mesh.with_inserted_attribute(ATTRIBUTE_BLEND_COLOR, blend_colors);

    commands.spawn((
        Mesh3d(meshes.add(mesh)),
        MeshMaterial3d(materials.add(CustomMaterial {
            color: LinearRgba::new(0.8, 1.0, 1.0, 1.0),
        })),
        Transform::from_xyz(0., 0.5, 0.),
    ));

    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
struct CustomMaterial {
    #[uniform(0)]
    color: LinearRgba,
}

impl Material for CustomMaterial {
    fn vertex_shader() -> bevy::shader::ShaderRef {
        SHADER_ASSET_PATH.into()
    }

    fn fragment_shader() -> bevy::shader::ShaderRef {
        SHADER_ASSET_PATH.into()
    }

    fn specialize(
        _pipeline: &bevy::pbr::MaterialPipeline,
        descriptor: &mut bevy::render::render_resource::RenderPipelineDescriptor,
        layout: &bevy::mesh::MeshVertexBufferLayoutRef,
        _key: bevy::pbr::MaterialPipelineKey<Self>,
    ) -> Result<(), bevy::render::render_resource::SpecializedMeshPipelineError> {
        let vertex_layout = layout.0.get_layout(&[
            Mesh::ATTRIBUTE_POSITION.at_shader_location(0),
            ATTRIBUTE_BLEND_COLOR.at_shader_location(1),
        ])?;
        descriptor.vertex.buffers = vec![vertex_layout];
        Ok(())
    }
}
