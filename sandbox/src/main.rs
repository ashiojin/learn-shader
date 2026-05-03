use bevy::prelude::*;

mod background;
mod camera;
mod config;
mod light;
mod meshes;
mod sample;

use background::BackgroundState;
use camera::SatelliteCamera;

use crate::{
    background::change_background,
    camera::{ZoomDirection, update_camera_follower},
    config::{ConfigState, draw_gizmo},
    light::{LightState, change_light, update_rotate_light},
    sample::{CustomMaterial, SampleState, change_sample},
};

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
        .insert_resource(ConfigState::default())
        .insert_resource(BackgroundState::default())
        .insert_resource(LightState::default())
        .add_systems(Startup, (setup,))
        .add_systems(Update, (react_to_keyevent, draw_gizmo))
        .add_systems(Update, update_camera_follower)
        .add_systems(Update, update_rotate_light)
        .add_systems(
            Update,
            change_sample.run_if(resource_changed::<SampleState>),
        )
        .add_systems(
            Update,
            change_background.run_if(resource_changed::<BackgroundState>),
        )
        .add_systems(Update, change_light.run_if(resource_changed::<LightState>))
        .run();
}

fn setup(mut commands: Commands) {
    // camera
    let satellite_camera = SatelliteCamera::new(2.5);
    commands.spawn((
        Camera3d::default(),
        satellite_camera.make_transform(),
        satellite_camera,
    ));
}

const SHADER_ASSET_PATH: &str = "shaders/fragment.wgsl";

#[allow(clippy::too_many_arguments)]
fn react_to_keyevent(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    asset_server: Res<AssetServer>,
    mut sample_state: ResMut<SampleState>,
    mut sattelite_camera: Single<(&mut SatelliteCamera, &mut Transform)>,
    mut other_state: ResMut<ConfigState>,
    mut background: ResMut<BackgroundState>,
    mut light_state: ResMut<LightState>,
) {
    // press N to switch to next sample
    if keys.just_pressed(KeyCode::KeyN) {
        sample_state.next_sample();
    }

    // press R to reload shader
    if keys.just_pressed(KeyCode::KeyR) {
        asset_server.reload(SHADER_ASSET_PATH);
    }

    // press WASD to rotate camera
    // press Z to zoom in, X to zoom out
    // press Q to reset camera
    if keys.any_pressed([
        KeyCode::KeyW,
        KeyCode::KeyA,
        KeyCode::KeyS,
        KeyCode::KeyD,
        KeyCode::KeyQ,
        KeyCode::KeyZ,
        KeyCode::KeyX,
    ]) {
        if keys.just_pressed(KeyCode::KeyQ) {
            sattelite_camera.0.reset();
        } else {
            let direction = if keys.pressed(KeyCode::KeyW) {
                Some(camera::RotateDirection::Up)
            } else if keys.pressed(KeyCode::KeyS) {
                Some(camera::RotateDirection::Down)
            } else if keys.pressed(KeyCode::KeyA) {
                Some(camera::RotateDirection::Left)
            } else if keys.pressed(KeyCode::KeyD) {
                Some(camera::RotateDirection::Right)
            } else {
                None
            };
            let zoom_direction = if keys.pressed(KeyCode::KeyZ) {
                Some(ZoomDirection::In)
            } else if keys.pressed(KeyCode::KeyX) {
                Some(ZoomDirection::Out)
            } else {
                None
            };
            if let Some(direction) = direction {
                sattelite_camera.0.rotate(direction, time.delta_secs());
            }
            if let Some(zoom_direction) = zoom_direction {
                sattelite_camera.0.zoom(zoom_direction, time.delta_secs());
            }
        }
        let new_transform = sattelite_camera.0.make_transform();
        sattelite_camera.1.clone_from(&new_transform);
    }

    // press b to toggle background
    if keys.just_pressed(KeyCode::KeyB) {
        background.next();
    }

    // press l to toggle light pattern
    if keys.just_pressed(KeyCode::KeyL) {
        light_state.next_pattern();
    }

    // press 1 to switch material
    if keys.just_pressed(KeyCode::Digit1) {
        sample_state.next_material();
    }

    // press 0 to toggle gizmo
    if keys.just_pressed(KeyCode::Digit0) {
        other_state.toggle_gizmo_cross();
    }
}
