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
            crate::WASM_RELOAD_REQUESTED.store(true, std::sync::atomic::Ordering::Relaxed);
            true
        }
        "CustomMaterial" => {
            crate::sample::material::write_custom_material(body);
            crate::WASM_RELOAD_REQUESTED.store(true, std::sync::atomic::Ordering::Relaxed);
            true
        }
        _ => false,
    }
}
