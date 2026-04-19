use std::f32::consts::PI;

use bevy::{color::palettes::css, prelude::*, render::render_resource::AsBindGroup};

fn main() {
    let asset_root_path = std::env::var("ASSETS_DIR").unwrap_or("assets".into());
    App::new()
        .add_plugins((
            DefaultPlugins
                .set(AssetPlugin {
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
            myshaderlib::MyShaderLibPlugin,
            MaterialPlugin::<CustomMaterial>::default(),
        ))
        .insert_resource(SampleState::default())
        .insert_resource(OtherState::default())
        .add_systems(Startup, (setup,))
        .add_systems(Update, (react_to_keyevent, draw_gizmo))
        .run();
}

#[derive(Component, Debug)]
struct SatelliteCamera {
    rotate_y: f32,
    rotate_x: f32,
    distance: f32,

    center: Vec3,
    up: Vec3,
}
impl SatelliteCamera {
    fn new(distance: f32) -> Self {
        Self {
            rotate_y: 0.0,
            rotate_x: 0.0,
            distance,
            center: Vec3::ZERO,
            up: Vec3::Y,
        }
    }

    fn make_transform(&self) -> Transform {
        let mut t = Transform::from_xyz(0.0, 0.0, self.distance);

        // rotate x around center
        t.rotate_around(self.center, Quat::from_rotation_x(self.rotate_x));
        // rotate y around center
        t.rotate_around(self.center, Quat::from_rotation_y(self.rotate_y));

        t.looking_at(self.center, self.up)
    }

    fn reset(&mut self) {
        self.rotate_y = 0.0;
        self.rotate_x = 0.0;
    }

    fn add_rotate_y(&mut self, delta: f32) {
        self.rotate_y += delta;
        // keep rotate_y in range [0, 2PI]
        if self.rotate_y > 2.0 * PI {
            self.rotate_y -= 2.0 * PI;
        } else if self.rotate_y < 0.0 {
            self.rotate_y += 2.0 * PI;
        }
    }

    fn add_rotate_x(&mut self, delta: f32) {
        self.rotate_x += delta;
        let ep = 0.01;
        // keep rotate_x in range [-PI/2 + ep, PI/2 - ep]
        if self.rotate_x > PI / 2.0 - ep {
            self.rotate_x = PI / 2.0 - ep;
        } else if self.rotate_x < -PI / 2.0 + ep {
            self.rotate_x = -PI / 2.0 + ep;
        }
    }
}

#[derive(Component, Debug, Clone)]
struct SampleMesh;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<CustomMaterial>>,
    mut sample_state: ResMut<SampleState>,
) {
    sample_state.update(&mut commands, &mut meshes, &mut materials);

    // camera
    let satellite_camera = SatelliteCamera::new(2.5);
    commands.spawn((
        Camera3d::default(),
        satellite_camera.make_transform(),
        satellite_camera,
    ));
}

#[derive(Debug, Clone, Default)]
enum SampleType {
    #[default]
    Plane,
    Cube,
    Cone,
}
impl SampleType {
    fn get_next(&self) -> Self {
        match self {
            SampleType::Plane => SampleType::Cube,
            SampleType::Cube => SampleType::Cone,
            SampleType::Cone => SampleType::Plane,
        }
    }
}

#[derive(Resource, Debug, Default)]
struct SampleState {
    sample_type: SampleType,
    entity: Option<Entity>,
}

impl SampleState {
    fn update(
        &mut self,
        commands: &mut Commands,
        meshes: &mut Assets<Mesh>,
        materials: &mut Assets<CustomMaterial>,
    ) {
        // despawn old sample
        if let Some(entity) = self.entity {
            commands.entity(entity).despawn();
        }

        let entity = match self.sample_type {
            SampleType::Plane => spawn_plane(commands, meshes, materials),
            SampleType::Cube => spawn_cube(commands, meshes, materials),
            SampleType::Cone => spawn_cone(commands, meshes, materials),
        };

        self.entity = Some(entity);
    }

    fn update_to_next(
        &mut self,
        commands: &mut Commands,
        meshes: &mut Assets<Mesh>,
        materials: &mut Assets<CustomMaterial>,
    ) {
        self.sample_type = self.sample_type.get_next();
        self.update(commands, meshes, materials);
    }
}

const SHADER_ASSET_PATH: &str = "shaders/fragment.wgsl";

#[allow(clippy::too_many_arguments)]
fn react_to_keyevent(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    asset_server: Res<AssetServer>,

    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<CustomMaterial>>,
    mut sample_state: ResMut<SampleState>,

    mut sattelite_camera: Single<(&mut SatelliteCamera, &mut Transform)>,

    mut other_state: ResMut<OtherState>,
) {
    // press N to switch to next sample
    if keys.just_pressed(KeyCode::KeyN) {
        sample_state.update_to_next(&mut commands, &mut meshes, &mut materials);
    }

    // press R to reload shader
    if keys.just_pressed(KeyCode::KeyR) {
        asset_server.reload(SHADER_ASSET_PATH);
    }

    // press WASD to rotate camera
    // press Q to reset camera
    if keys.any_pressed([
        KeyCode::KeyW,
        KeyCode::KeyA,
        KeyCode::KeyS,
        KeyCode::KeyD,
        KeyCode::KeyQ,
    ]) {
        info!("trans : {:?}", sattelite_camera.1);
        if keys.just_pressed(KeyCode::KeyQ) {
            sattelite_camera.0.reset();
        } else {
            let delta = time.delta_secs() * PI;
            if keys.pressed(KeyCode::KeyW) {
                sattelite_camera.0.add_rotate_x(delta);
            }
            if keys.pressed(KeyCode::KeyS) {
                sattelite_camera.0.add_rotate_x(-delta);
            }
            if keys.pressed(KeyCode::KeyA) {
                sattelite_camera.0.add_rotate_y(delta);
            }
            if keys.pressed(KeyCode::KeyD) {
                sattelite_camera.0.add_rotate_y(-delta);
            }
        }
        let new_transform = sattelite_camera.0.make_transform();
        sattelite_camera.1.clone_from(&new_transform);
        info!("->trans : {:?}", sattelite_camera.1);
    }

    // press 0 to toggle gizmo
    if keys.just_pressed(KeyCode::Digit0) {
        other_state.gizmo_cross = !other_state.gizmo_cross;
    }
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
struct CustomMaterial {}

impl Material for CustomMaterial {
    fn fragment_shader() -> bevy::shader::ShaderRef {
        SHADER_ASSET_PATH.into()
    }
}

fn spawn_plane(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<CustomMaterial>,
) -> Entity {
    // plane facing to camera
    commands
        .spawn((
            Mesh3d(meshes.add(Plane3d::new(Vec3::Z, Vec2::new(1.0, 1.0)))),
            MeshMaterial3d(materials.add(CustomMaterial {})),
            Transform::from_xyz(0., 0., 0.),
            SampleMesh,
        ))
        .id()
}

fn spawn_cube(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<CustomMaterial>,
) -> Entity {
    // TODO: UV mapping of cone is unsuitable for our purpose. We can implement our `Meshable` for
    // it.
    commands
        .spawn((
            Mesh3d(meshes.add(Cuboid::from_length(1.0))),
            MeshMaterial3d(materials.add(CustomMaterial {})),
            Transform::from_xyz(0., 0., 0.),
            SampleMesh,
        ))
        .id()
}

fn spawn_cone(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<CustomMaterial>,
) -> Entity {
    // TODO: UV mapping of cone is unsuitable for our purpose. We can implement our `Meshable` for
    // it.
    commands
        .spawn((
            Mesh3d(meshes.add(Cone::new(0.5, 1.0))),
            MeshMaterial3d(materials.add(CustomMaterial {})),
            Transform::from_xyz(0., 0., 0.),
            SampleMesh,
        ))
        .id()
}

#[derive(Resource, Debug, Default)]
struct OtherState {
    gizmo_cross: bool,
}

fn draw_gizmo(
    mut gizmos: Gizmos,
    other_state: Res<OtherState>,
    sample_mesh: Query<&Transform, With<SampleMesh>>,
) {
    if other_state.gizmo_cross {
        for transform in sample_mesh.iter() {
            let pos = transform.translation;
            gizmos.line(pos - Vec3::X, pos + Vec3::X, css::RED);
            gizmos.line(pos - Vec3::Y, pos + Vec3::Y, css::GREEN);
            gizmos.line(pos - Vec3::Z, pos + Vec3::Z, css::BLUE);
        }
    }
}
