use bevy::{
    color::palettes::css::RED,
    pbr::{ExtendedMaterial, MaterialExtension},
    prelude::*,
    render::render_resource::AsBindGroup,
};



fn main() {
    let asset_root_path = std::env::var("ASSETS_DIR").unwrap_or("assets".into());
    App::new()
        .add_plugins((
            DefaultPlugins.set(AssetPlugin {
                file_path: asset_root_path,
                ..Default::default()
            }),
            MaterialPlugin::<ExtendedMaterial<StandardMaterial, MyExtension>>::default(),
        ))
        .add_systems(Startup, (setup,))
        .add_systems(Update, (rotate_things,))
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ExtendedMaterial<StandardMaterial, MyExtension>>>,
    mut standard_materials: ResMut<Assets<StandardMaterial>>,
) {
    // extended
    commands.spawn((
        Mesh3d(meshes.add(Sphere::new(1.0))),
        MeshMaterial3d(materials.add(ExtendedMaterial {
            base: StandardMaterial {
                base_color: RED.into(),
                opaque_render_method: bevy::pbr::OpaqueRendererMethod::Auto,
                ..default()
            },
            extension: MyExtension::new(7),
        })),
        // MeshMaterial3d(standard_materials.add(StandardMaterial {
        //     base_color: GREEN.into(),
        //     opaque_render_method: bevy::pbr::OpaqueRendererMethod::Auto,
        //     ..default()
        // })),
        Transform::from_xyz(-1.0, 0.5, 0.0),
    ));

    // standard for comparison
    commands.spawn((
        Mesh3d(meshes.add(Sphere::new(1.0))),
        MeshMaterial3d(standard_materials.add(StandardMaterial {
            base_color: RED.into(),
            opaque_render_method: bevy::pbr::OpaqueRendererMethod::Auto,
            ..default()
        })),
        Transform::from_xyz(1.0, 0.5, 0.0),
        Rotate,
    ));

    // light
    commands.spawn((
        DirectionalLight::default(),
        Transform::from_xyz(2.0, 2.0, 2.0).looking_at(Vec3::ZERO, Vec3::Y),
        Rotate,
    ));

    // camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 2.5, 6.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

#[derive(Component, Debug)]
struct Rotate;

fn rotate_things(mut q: Query<&mut Transform, With<Rotate>>, time: Res<Time>) {
    for mut t in &mut q {
        t.rotate_y(time.delta_secs());
    }
}

#[derive(Asset, AsBindGroup, Reflect, Debug, Clone, Default)]
struct MyExtension {
    #[uniform(100)]
    x: u32,
    #[cfg(feature = "webgl2")]
    #[uniform(100)]
    _webgl2_padding_8b: u32,
    #[cfg(feature = "webgl2")]
    #[uniform(100)]
    _webgl2_padding_12b: u32,
    #[cfg(feature = "webgl2")]
    #[uniform(100)]
    _webgl2_padding_16b: u32,
}
impl MyExtension {
    fn new(x: u32) -> Self {
        Self {
            x,
            ..default()
        }
    }
}

const SHADER_ASSET_PATH: &str = "shaders/my_material.wgsl";
impl MaterialExtension for MyExtension {
    fn fragment_shader() -> bevy::shader::ShaderRef {
        SHADER_ASSET_PATH.into()
    }

    fn deferred_fragment_shader() -> bevy::shader::ShaderRef {
        SHADER_ASSET_PATH.into()
    }
}
