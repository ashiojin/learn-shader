#![cfg(target_arch = "wasm32")]

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn read_wgsl(shader_type: &str) -> Option<String> {
    match shader_type {
        "ExtendedMaterial" => Some(crate::sample::extended_material::read_global_res()),
        "CustomMaterial" => Some(crate::sample::material::read_custom_material()),
        _ => None,
    }
}

#[wasm_bindgen]
pub fn write_wgsl(shader_type: &str, body: &str) -> bool {
    match shader_type {
        "ExtendedMaterial" => {
            crate::sample::extended_material::write_global_res(body);
            crate::api_shared::send_command(crate::api_shared::ApiCommand::Reload);
            true
        }
        "CustomMaterial" => {
            crate::sample::material::write_custom_material(body);
            crate::api_shared::send_command(crate::api_shared::ApiCommand::Reload);
            true
        }
        _ => false,
    }
}

#[wasm_bindgen]
pub fn send_command_js(cmd_json: &str) -> bool {
    if let Ok(command) = serde_json::from_str::<crate::api_shared::ApiCommand>(cmd_json) {
        crate::api_shared::send_command(command);
        true
    } else {
        false
    }
}

#[wasm_bindgen]
pub fn get_status_js() -> Option<String> {
    let status = crate::api_shared::get_app_status();
    serde_json::to_string(&status).ok()
}

#[wasm_bindgen]
pub fn reload_shaders_js() {
    crate::api_shared::send_command(crate::api_shared::ApiCommand::Reload);
}

#[wasm_bindgen]
pub fn select_mode_js(mode: &str) {
    crate::api_shared::send_command(crate::api_shared::ApiCommand::SelectMode(mode.to_string()));
}
