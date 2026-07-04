use bevy::prelude::*;

pub mod api;

pub mod billboard;
mod background;
mod camera;
mod config;
mod light;
mod random;
mod sample;

use camera::SatelliteCamera;

use crate::{
    api::UnifiedApiPlugin,
    background::BackgroundPlugin,
    billboard::BillboardPlugin,
    config::DebugGizmoPlugin,
    light::LightPlugin,
    random::RandomPlugin,
    sample::{SamplePlugin, SampleState},
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
            DebugGizmoPlugin,
            BillboardPlugin,
            UnifiedApiPlugin,
        ))
        .add_systems(Startup, (setup,))
        .add_systems(
            Update,
            (
                react_to_keyevent,
            ),
        )
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


fn react_to_keyevent(
    keys: Res<ButtonInput<KeyCode>>,
    mut sample_state: ResMut<SampleState>,
) {
    handle_sample_input(&keys, &mut sample_state);
}
