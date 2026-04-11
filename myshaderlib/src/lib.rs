use bevy::{
    asset::{embedded_asset},
    prelude::*,
};

pub struct MyShaderLibPlugin;

#[derive(Resource)]
struct MyShaderLibShader {
    _lib: Handle<Shader>,
    _simplex_noise: Handle<Shader>,
}
impl MyShaderLibShader {
    fn new(lib: Handle<Shader>, simplex_noise: Handle<Shader>) -> Self {
        Self {
            _lib: lib,
            _simplex_noise: simplex_noise,
        }
    }
}


impl Plugin for MyShaderLibPlugin {
    fn build(&self, app: &mut App) {
        embedded_asset!(app, "lib.wgsl");
        embedded_asset!(app, "simplex_noise.wgsl");
        let asset_server = app.world_mut().resource_mut::<AssetServer>();
        let lib = asset_server.load::<Shader>("embedded://myshaderlib/lib.wgsl");
        let simplex_noise = asset_server.load::<Shader>("embedded://myshaderlib/simplex_noise.wgsl");
        app.insert_resource(MyShaderLibShader::new(lib, simplex_noise));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}
