use bevy::{prelude::*, render::render_resource::AsBindGroup};

fn main() {
    let asset_root_path = std::env::var("ASSETS_DIR").unwrap_or("assets".into());
    App::new()
        .add_plugins((
            DefaultPlugins.set(AssetPlugin {
                file_path: asset_root_path,
                //watch_for_changes_override: Some(true),
                ..Default::default()
            })
            .set(WindowPlugin {
                primary_window: Some(Window {
                    resolution: (640, 640).into(),
                    resize_constraints: WindowResizeConstraints {
                        min_width: 640.0,
                        min_height: 640.0,
                        max_width: 640.0,
                        max_height: 640.0,
                    },
                    ..default()
                }),
                ..default()
            }),
            MaterialPlugin::<CustomMaterial>::default(),
        ))
        .add_systems(Startup, (setup,))
        .add_systems(Update, (refresh_shader,))
        .run();
}

#[derive(Component, Debug, Clone)]
struct PlaneForShader;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<CustomMaterial>>,
) {
    // plane facing to camera
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::new(Vec3::Z, Vec2::new(5.0, 5.0)))),
        MeshMaterial3d(materials.add(CustomMaterial {})),
        Transform::from_xyz(0., 0., 0.),
        PlaneForShader,
    ));

    // camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 0.0, 12.5).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

const SHADER_ASSET_PATH: &str = "shaders/plane_frag.wgsl";
fn refresh_shader(
    keys : Res<ButtonInput<KeyCode>>,
    asset_server: Res<AssetServer>,
) {
    if !keys.just_pressed(KeyCode::KeyR) {
        return;
    }

    asset_server.reload(SHADER_ASSET_PATH);
}



#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
struct CustomMaterial {}

impl Material for CustomMaterial {
    fn fragment_shader() -> bevy::shader::ShaderRef {
        SHADER_ASSET_PATH.into()
    }
}
