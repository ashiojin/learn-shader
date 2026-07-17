use bevy::asset::RecursiveDependencyLoadState;
use bevy::gltf::Gltf;
use bevy::prelude::*;

use super::state::SampleType;

/// Whether the app is still loading GLTF assets or ready to run.
#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AppState {
    #[default]
    Loading,
    Running,
}

/// Holds strong handles to every pre-loaded GLTF asset for the whole app
/// lifetime, so the assets (and their sub-assets: scenes, meshes, animation
/// clips) are never unloaded and never need reloading.
#[derive(Resource, Default)]
pub struct PreloadedGltf {
    pub handles: Vec<Handle<Gltf>>,
}

pub struct PreloadPlugin;

impl Plugin for PreloadPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<AppState>()
            .add_systems(Startup, preload_gltf_assets)
            .add_systems(
                Update,
                check_preload_complete.run_if(
                    in_state(AppState::Loading).and_then(resource_exists::<PreloadedGltf>),
                ),
            );
    }
}

/// Startup: derive the distinct GLTF file paths from `SampleType`, load each as
/// a whole `Gltf` asset, and retain the strong handles in `PreloadedGltf`.
fn preload_gltf_assets(mut commands: Commands, asset_server: Res<AssetServer>) {
    // BTreeSet dedups paths so two SampleTypes sharing a file load only once.
    let paths: std::collections::BTreeSet<&'static str> = SampleType::all_variants()
        .iter()
        .filter_map(|s| s.gltf_asset())
        .map(|a| a.path)
        .collect();

    let handles: Vec<Handle<Gltf>> = paths
        .into_iter()
        .map(|path| asset_server.load::<Gltf>(path.to_string()))
        .collect();

    info!("Preloading {} GLTF asset(s)", handles.len());
    commands.insert_resource(PreloadedGltf { handles });
}

/// Update (while Loading): once every retained handle has settled
/// (Loaded or Failed), transition to Running. Failed assets are logged but do
/// not block the transition, so a bad asset can't hang the app.
fn check_preload_complete(
    asset_server: Res<AssetServer>,
    preloaded: Res<PreloadedGltf>,
    mut next: ResMut<NextState<AppState>>,
) {
    let mut all_settled = true;
    for handle in &preloaded.handles {
        match asset_server.recursive_dependency_load_state(handle) {
            RecursiveDependencyLoadState::Loaded => {}
            RecursiveDependencyLoadState::Failed(err) => {
                warn!("GLTF preload failed: {err:?}");
            }
            RecursiveDependencyLoadState::NotLoaded | RecursiveDependencyLoadState::Loading => {
                all_settled = false;
            }
        }
    }

    if all_settled {
        info!("GLTF preload complete; entering Running state");
        next.set(AppState::Running);
    }
}
