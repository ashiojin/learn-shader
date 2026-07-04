#![cfg(target_arch = "wasm32")]

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn read_wgsl(shader_type: &str) -> Option<String> {
    crate::api_shared::read_wgsl(shader_type)
}

#[wasm_bindgen]
pub fn write_wgsl(shader_type: &str, body: &str) -> bool {
    crate::api_shared::write_wgsl(shader_type, body)
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
pub fn select_sample_mode_js(mode: &str) {
    crate::api_shared::send_command(crate::api_shared::ApiCommand::SelectSampleMode(mode.to_string()));
}

#[wasm_bindgen]
pub fn select_material_mode_js(mode: &str) {
    crate::api_shared::send_command(crate::api_shared::ApiCommand::SelectMaterialMode(mode.to_string()));
}
