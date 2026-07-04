use bevy::prelude::*;

pub mod api;

mod background;
mod camera;
mod config;
mod light;
mod random;
mod sample;

use camera::SatelliteCamera;

use crate::{
    background::BackgroundPlugin,
    config::{ConfigState, draw_gizmo, draw_xy_grid_gizmo},
    light::LightPlugin,
    random::RandomPlugin,
    sample::{
        SamplePlugin, SampleState, extended_material::ReloadReq,
    },
};

pub mod api_shared;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn start() {
    run_app();
}

fn main() {
    #[cfg(not(target_arch = "wasm32"))]
    run_app();
}

fn run_app() {
    api::spawn_api_server();

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
            camera::SatelliteCameraPlugin,
            BackgroundPlugin,
            LightPlugin,
        ))
        .insert_resource(ConfigState::default())
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
        .run();
}

fn poll_api_commands(
    mut reload_reqs: MessageWriter<ReloadReq>,
    mut sample_state: Option<ResMut<SampleState>>,
    mut custom_shader: ResMut<crate::sample::material::CustomMaterialShader>,
    mut extended_shader: ResMut<crate::sample::extended_material::ExtendedMaterialShader>,
) {
    let mut reload = false;
    for cmd in crate::api_shared::pop_commands() {
        match cmd {
            crate::api_shared::ApiCommand::Reload => {
                reload = true;
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
    if reload {
        if let Some(src) = crate::api_shared::read_wgsl("CustomMaterial") {
            custom_shader.0 = src;
        }
        if let Some(src) = crate::api_shared::read_wgsl("ExtendedMaterial") {
            extended_shader.0 = src;
        }
        reload_reqs.write(ReloadReq);
        info!("Reload requested via unified API");
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


fn handle_gizmo_input(
    keys: &ButtonInput<KeyCode>,
    other_state: &mut ResMut<ConfigState>,
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

fn react_to_keyevent(
    keys: Res<ButtonInput<KeyCode>>,
    mut sample_state: ResMut<SampleState>,
    mut other_state: ResMut<ConfigState>,
) {
    handle_sample_input(&keys, &mut sample_state);
    handle_gizmo_input(&keys, &mut other_state);
}
