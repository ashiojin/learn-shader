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
