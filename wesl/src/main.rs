use bevy::{math::VectorSpace, prelude::*, render::render_resource::AsBindGroup, shader::ShaderDefVal};

fn main() {
    let asset_root_path = std::env::var("ASSETS_DIR").unwrap_or("assets".into());
    App::new()
        .add_plugins((
            DefaultPlugins.set(AssetPlugin {
                file_path: asset_root_path,
                ..Default::default()
            }),
            MaterialPlugin::<CustomMaterial>::default(),
            CustomMaterialPlugin,
        ))
        .add_systems(Startup, (setup,))
        .add_systems(Update, (update,))
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<CustomMaterial>>,
) {
    // cube
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::default())),
        MeshMaterial3d(materials.add(CustomMaterial {
            time: Vec4::ZERO,
            party_mode: false,
        })),
        Transform::from_xyz(0.0, 0.5, 0.0),
    ));

    // camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}
fn update(
    time: Res<Time>,
    mut query: Query<(&MeshMaterial3d<CustomMaterial>, &mut Transform)>,
    mut materials: ResMut<Assets<CustomMaterial>>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    for (material, mut transform) in &mut query {
        let material = materials.get_mut(material).unwrap();
        material.time.x = time.elapsed_secs();
        if keys.just_pressed(KeyCode::Space) {
            material.party_mode = !material.party_mode;
        }

        if material.party_mode {
            transform.rotate(Quat::from_rotation_y(0.005));
        }
    }
}

const FRAGMENT_SHADER_ASSET_PATH: &str = "shaders/custom_material.wesl";

pub struct CustomMaterialPlugin;

impl Plugin for CustomMaterialPlugin {
    fn build(&self, app: &mut App) {
        #[cfg(not(feature = "disable_pre_load"))]
        {
            let handle = app
                .world_mut()
                .resource_mut::<AssetServer>()
                .load::<Shader>("shaders/util.wesl");
            app.insert_resource(UtilityShader(handle));
        }
    }
}

#[derive(Resource)]
struct UtilityShader(Handle<Shader>);

#[derive(Asset, TypePath, AsBindGroup, Clone)]
#[bind_group_data(CustomMaterialKey)]
struct CustomMaterial {
    #[uniform(0)]
    time: Vec4,
    party_mode: bool,
}

#[repr(C)]
#[derive(Eq, PartialEq, Hash, Copy, Clone)]
struct CustomMaterialKey {
    party_mode: bool,
}

impl From<&CustomMaterial> for CustomMaterialKey {
    fn from(value: &CustomMaterial) -> Self {
        Self {
            party_mode: value.party_mode,
        }
    }
}

impl Material for CustomMaterial {
    fn fragment_shader() -> bevy::shader::ShaderRef {
        FRAGMENT_SHADER_ASSET_PATH.into()
    }

    fn specialize(
        pipeline: &bevy::pbr::MaterialPipeline,
        descriptor: &mut bevy::render::render_resource::RenderPipelineDescriptor,
        layout: &bevy::mesh::MeshVertexBufferLayoutRef,
        key: bevy::pbr::MaterialPipelineKey<Self>,
    ) -> Result<(), bevy::render::render_resource::SpecializedMeshPipelineError> {
        let fragment = descriptor.fragment.as_mut().unwrap();
        fragment.shader_defs.push(ShaderDefVal::Bool(
            "PARTY_MODE".to_string(),
            key.bind_group_data.party_mode,
        ));
        Ok(())
    }
}
