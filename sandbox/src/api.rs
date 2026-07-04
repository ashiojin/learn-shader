use bevy::prelude::*;

#[cfg(not(target_arch = "wasm32"))]
pub mod native;

#[cfg(target_arch = "wasm32")]
pub mod wasm;

pub fn spawn_api_server() {
    #[cfg(not(target_arch = "wasm32"))]
    {
        std::thread::spawn(move || {
            native::spawn_api_server();
        });
    }
}

pub fn poll_api_commands(
    mut reload_reqs: MessageWriter<crate::sample::extended_material::ReloadReq>,
    mut sample_state: Option<ResMut<crate::sample::SampleState>>,
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
        reload_reqs.write(crate::sample::extended_material::ReloadReq);
        info!("Reload requested via unified API");
    }
}

pub fn sync_telemetry_cache(sample_state: Option<Res<crate::sample::SampleState>>) {
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

pub struct UnifiedApiPlugin;

impl Plugin for UnifiedApiPlugin {
    fn build(&self, app: &mut App) {
        spawn_api_server();
        app.add_systems(
            Update,
            (
                poll_api_commands,
                sync_telemetry_cache,
            ),
        );
    }
}
