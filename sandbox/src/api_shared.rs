use std::sync::{OnceLock, Mutex, RwLock};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum ApiCommand {
    Reload,
    SelectSampleMode(String),
    SelectMaterialMode(String),
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
pub struct AppStatus {
    pub current_sample_mode: String,
    pub available_sample_modes: Vec<String>,
    pub current_material_mode: String,
    pub available_material_modes: Vec<String>,
}

static COMMAND_QUEUE: OnceLock<Mutex<Vec<ApiCommand>>> = OnceLock::new();
static APP_STATUS: OnceLock<RwLock<AppStatus>> = OnceLock::new();

/// Push a command to the queue from any thread/runtime.
pub fn send_command(command: ApiCommand) {
    let queue = COMMAND_QUEUE.get_or_init(|| Mutex::new(Vec::new()));
    if let Ok(mut guard) = queue.lock() {
        guard.push(command);
    }
}

/// Pop and drain all queued commands to be processed in the next Bevy frame.
pub fn pop_commands() -> Vec<ApiCommand> {
    let queue = COMMAND_QUEUE.get_or_init(|| Mutex::new(Vec::new()));
    if let Ok(mut guard) = queue.lock() {
        std::mem::take(&mut *guard)
    } else {
        Vec::new()
    }
}

/// Retrieve the current status cache.
pub fn get_app_status() -> AppStatus {
    APP_STATUS.get_or_init(|| RwLock::new(AppStatus::default()))
        .read()
        .map(|guard| guard.clone())
        .unwrap_or_default()
}

/// Update the current status cache from Bevy.
pub fn update_app_status(status: AppStatus) {
    if let Ok(mut guard) = APP_STATUS.get_or_init(|| RwLock::new(AppStatus::default())).write() {
        *guard = status;
    }
}

/// Read WGSL shader text by type name.
pub fn read_wgsl(shader_type: &str) -> Option<String> {
    match shader_type {
        "ExtendedMaterial" => Some(crate::sample::extended_material::read_global_res()),
        "CustomMaterial" => Some(crate::sample::material::read_custom_material()),
        _ => None,
    }
}

/// Write WGSL shader text by type name and trigger a reload.
pub fn write_wgsl(shader_type: &str, body: &str) -> bool {
    match shader_type {
        "ExtendedMaterial" => {
            crate::sample::extended_material::write_global_res(body);
            send_command(ApiCommand::Reload);
            true
        }
        "CustomMaterial" => {
            crate::sample::material::write_custom_material(body);
            send_command(ApiCommand::Reload);
            true
        }
        _ => false,
    }
}
