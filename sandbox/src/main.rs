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
    config::{ConfigState, draw_gizmo},
    light::{LightState, change_light, update_rotate_light},
    random::RandomPlugin,
    sample::{SamplePlugin, SampleState, extended_material::ReloadReq, init_globals, reload_shaders},
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
    let default_plugin =
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
                default_plugin
                ,
            myshaderlib::MyShaderLibPlugin,
            RandomPlugin,
            SamplePlugin,
        ))
        .insert_resource(ConfigState::default())
        .insert_resource(BackgroundState::default())
        .insert_resource(LightState::default())
        .add_systems(Startup, (setup,))
        .add_systems(Update, (react_to_keyevent, draw_gizmo, poll_api_commands, sync_telemetry_cache))
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
                    match mode.as_str() {
                        "Saru" => state.sample_type = crate::sample::state::SampleType::Saru,
                        "Plane" => state.sample_type = crate::sample::state::SampleType::Plane,
                        "Cube" => state.sample_type = crate::sample::state::SampleType::Cube,
                        "Cone" => state.sample_type = crate::sample::state::SampleType::Cone,
                        "Sphere" => state.sample_type = crate::sample::state::SampleType::Sphere,
                        "Ring" => state.sample_type = crate::sample::state::SampleType::Ring,
                        "SphericalZone" => state.sample_type = crate::sample::state::SampleType::SphericalZone,
                        "Belt" => state.sample_type = crate::sample::state::SampleType::Belt,
                        "Emitter1" => state.sample_type = crate::sample::state::SampleType::Emitter1,
                        _ => warn!("Unknown sample mode requested via API: {}", mode),
                    }
                }
            }
            crate::api_shared::ApiCommand::SelectMaterialMode(mode) => {
                info!("API requested material mode selection: {}", mode);
                if let Some(state) = sample_state.as_deref_mut() {
                    match mode.as_str() {
                        "User" => state.material_type = crate::sample::state::SampleMaterialType::User,
                        "UserExtended" => state.material_type = crate::sample::state::SampleMaterialType::UserExtended,
                        "UvTest1024" => state.material_type = crate::sample::state::SampleMaterialType::UvTest1024,
                        _ => warn!("Unknown material mode requested via API: {}", mode),
                    }
                }
            }
        }
    }
}

fn sync_telemetry_cache(sample_state: Option<Res<SampleState>>) {
    if let Some(state) = sample_state {
        let current_sample_mode = format!("{:?}", state.sample_type);
        let available_sample_modes = vec![
            "Saru".to_string(),
            "Plane".to_string(),
            "Cube".to_string(),
            "Cone".to_string(),
            "Sphere".to_string(),
            "Ring".to_string(),
            "SphericalZone".to_string(),
            "Belt".to_string(),
            "Emitter1".to_string(),
        ];
        let current_material_mode = format!("{:?}", state.material_type);
        let available_material_modes = vec![
            "User".to_string(),
            "UserExtended".to_string(),
            "UvTest1024".to_string(),
        ];
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
        reload_shaders(&asset_server);
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

    // press 2 to toggle billboard
    if keys.just_pressed(KeyCode::Digit2) {
        sample_state.toggle_billboard();
    }

    // press 0 to toggle gizmo
    if keys.just_pressed(KeyCode::Digit0) {
        other_state.toggle_gizmo_cross();
    }
}
