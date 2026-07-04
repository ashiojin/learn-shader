use bevy::prelude::*;

#[cfg(not(target_arch = "wasm32"))]
mod api;

#[cfg(target_arch = "wasm32")]
pub mod wasm_api;

mod background;
mod camera;
mod config;
mod light;
mod random;
mod sample;

use background::BackgroundState;
use camera::SatelliteCamera;

use crate::{
    background::change_background,
    camera::{ZoomDirection, update_camera_follower},
    config::{ConfigState, draw_gizmo, draw_xy_grid_gizmo},
    light::{LightState, change_light, update_rotate_light},
    random::RandomPlugin,
    sample::{
        SamplePlugin, SampleState, extended_material::ReloadReq, init_globals,
    },
};

pub mod api_shared;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn start() {
    run_app();
}

fn main() {
    init_globals();
    #[cfg(not(target_arch = "wasm32"))]
    run_app();
}

fn run_app() {
    #[cfg(not(target_arch = "wasm32"))]
    std::thread::spawn(move || {
        api::spawn_api_server();
    });

    let asset_root_path = std::env::var("ASSETS_DIR").unwrap_or("assets".into());
    let default_plugin = DefaultPlugins
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
                canvas: Some("#bevy".to_owned()), // only used for wasm, but doesn't hurt to set it for native
                fit_canvas_to_parent: true, // only used for wasm, but doesn't hurt to set it for native
                ..default()
            }),
            ..default()
        })
        .build();
    // #[cfg(target_family = "wasm")]
    // let default_plugin = default_plugin.disable::<LogPlugin>();

    App::new()
        .add_plugins((
            default_plugin,
            myshaderlib::MyShaderLibPlugin,
            RandomPlugin,
            SamplePlugin,
        ))
        .insert_resource(ConfigState::default())
        .insert_resource(BackgroundState::default())
        .insert_resource(LightState::default())
        .add_systems(Startup, (setup,))
        .add_systems(
            Update,
            (
                react_to_keyevent,
                draw_gizmo,
                draw_xy_grid_gizmo,
                poll_api_commands,
                sync_telemetry_cache,
            ),
        )
        .add_systems(Update, update_camera_follower)
        .add_systems(Update, update_rotate_light)
        .add_systems(
            Update,
            change_background.run_if(resource_changed::<BackgroundState>),
        )
        .add_systems(Update, change_light.run_if(resource_changed::<LightState>))
        .run();
}

fn poll_api_commands(
    mut reload_reqs: MessageWriter<ReloadReq>,
    mut sample_state: Option<ResMut<SampleState>>,
) {
    for cmd in crate::api_shared::pop_commands() {
        match cmd {
            crate::api_shared::ApiCommand::Reload => {
                reload_reqs.write(ReloadReq);
                info!("Reload requested via unified API");
            }
            crate::api_shared::ApiCommand::SelectSampleMode(mode) => {
                info!("API requested sample mode selection: {}", mode);
                if let Some(state) = sample_state.as_deref_mut() {
                    if let Some(sample_type) = crate::sample::state::SampleType::from_str(&mode) {
                        state.sample_type = sample_type;
                    } else {
                        warn!("Unknown sample mode requested via API: {}", mode);
                    }
                }
            }
            crate::api_shared::ApiCommand::SelectMaterialMode(mode) => {
                info!("API requested material mode selection: {}", mode);
                if let Some(state) = sample_state.as_deref_mut() {
                    if let Some(material_type) = crate::sample::state::SampleMaterialType::from_str(&mode) {
                        state.material_type = material_type;
                    } else {
                        warn!("Unknown material mode requested via API: {}", mode);
                    }
                }
            }
        }
    }
}

fn sync_telemetry_cache(sample_state: Option<Res<SampleState>>) {
    if let Some(state) = sample_state {
        let current_sample_mode = state.sample_type.as_str().to_string();
        let available_sample_modes = crate::sample::state::SampleType::all_variants()
            .iter()
            .map(|v| v.as_str().to_string())
            .collect();
        let current_material_mode = state.material_type.as_str().to_string();
        let available_material_modes = crate::sample::state::SampleMaterialType::all_variants()
            .iter()
            .map(|v| v.as_str().to_string())
            .collect();
        crate::api_shared::update_app_status(crate::api_shared::AppStatus {
            current_sample_mode,
            available_sample_modes,
            current_material_mode,
            available_material_modes,
        });
    }
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

fn handle_sample_input(
    keys: &ButtonInput<KeyCode>,
    sample_state: &mut ResMut<SampleState>,
) {
    // press N to switch to next sample
    if keys.just_pressed(KeyCode::KeyN) {
        sample_state.next_sample();
    }

    // press R to respawn emitter and models
    if keys.just_pressed(KeyCode::KeyR) {
        sample_state.set_changed();
    }

    // press 1 to switch material
    if keys.just_pressed(KeyCode::Digit1) {
        sample_state.next_material();
    }

    // press 2 to toggle billboard
    if keys.just_pressed(KeyCode::Digit2) {
        sample_state.toggle_billboard();
    }
}

fn handle_camera_input(
    keys: &ButtonInput<KeyCode>,
    time: &Time,
    camera: &mut SatelliteCamera,
    transform: &mut Transform,
) {
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
            camera.reset();
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
                camera.rotate(direction, time.delta_secs());
            }
            if let Some(zoom_direction) = zoom_direction {
                camera.zoom(zoom_direction, time.delta_secs());
            }
        }
        let new_transform = camera.make_transform();
        transform.clone_from(&new_transform);
    }
}

fn handle_background_input(
    keys: &ButtonInput<KeyCode>,
    background: &mut BackgroundState,
) {
    // press b to toggle background
    if keys.just_pressed(KeyCode::KeyB) {
        background.next();
    }
}

fn handle_light_input(
    keys: &ButtonInput<KeyCode>,
    light_state: &mut LightState,
) {
    // press l to toggle light pattern
    if keys.just_pressed(KeyCode::KeyL) {
        light_state.next_pattern();
    }
}

fn handle_gizmo_input(
    keys: &ButtonInput<KeyCode>,
    other_state: &mut ConfigState,
) {
    // press 0 to toggle gizmo
    if keys.just_pressed(KeyCode::Digit0) {
        other_state.toggle_gizmo_cross();
    }

    // press 9 to toggle gizmo for debug
    if keys.just_pressed(KeyCode::Digit9) {
        other_state.toggle_gizmo_for_debug();
    }
}

#[allow(clippy::too_many_arguments)]
fn react_to_keyevent(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut sample_state: ResMut<SampleState>,
    mut sattelite_camera: Single<(&mut SatelliteCamera, &mut Transform)>,
    mut other_state: ResMut<ConfigState>,
    mut background: ResMut<BackgroundState>,
    mut light_state: ResMut<LightState>,
) {
    handle_sample_input(&keys, &mut sample_state);

    let (camera, transform) = &mut *sattelite_camera;
    handle_camera_input(&keys, &time, camera, transform);

    handle_background_input(&keys, &mut background);
    handle_light_input(&keys, &mut light_state);
    handle_gizmo_input(&keys, &mut other_state);
}
