use bevy::prelude::*;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about = "A POC to load Gltf from any path", long_about = None)]
struct Args {
    /// The path to the GLTF file to load
    #[arg(short, long)]
    path: String,
}

#[derive(Resource, Debug)]
struct GltfPath(String);

#[derive(Resource, Debug)]
struct Pwd(String);

#[derive(Resource, Debug)]
struct GltfHandle(Handle<Gltf>);

fn main() {
    let args = Args::parse();

    let gltf_path = GltfPath(args.path);
    let pwd: String = std::env::current_dir()
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();

    let pwd_res = Pwd(pwd.clone());
    info!("Current working directory: {}", pwd);
    info!("GLTF path: {}", gltf_path.0);

    App::new()
        .add_plugins(DefaultPlugins.set(AssetPlugin {
            file_path: pwd,
            unapproved_path_mode: bevy::asset::UnapprovedPathMode::Allow,
            ..default()
        }))
        .insert_resource(gltf_path)
        .insert_resource(pwd_res)
        .add_systems(Startup, setup)
        .add_systems(Update, load_gltf.run_if(resource_changed::<GltfPath>))
        .add_systems(Update, spawn_gltf.run_if(resource_exists::<GltfHandle>))
        .run();
}

fn setup(mut commands: Commands) {
    // camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0., 5., 10.).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

fn load_gltf(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    gltf_path: Res<GltfPath>,
    q_gltf_spawned: Query<Entity, With<GltfSpawned>>,
) {
    info!("Loading GLTF from path: {}", gltf_path.0);
    commands.insert_resource(GltfHandle(asset_server.load(gltf_path.0.clone())));
    for entity in q_gltf_spawned.iter() {
        commands.entity(entity).try_despawn();
    }
}

#[derive(Component, Debug)]
struct GltfSpawned;

fn spawn_gltf(
    mut commands: Commands,
    gltf_handle: Res<GltfHandle>,
    gltf_assets: Res<Assets<Gltf>>,
    asset_server: Res<AssetServer>,
    q_gltf_spawned: Query<Entity, With<GltfSpawned>>
) {
    if q_gltf_spawned.iter().count() > 0 {
        return;
    }
    if let Some(gltf) = gltf_assets.get(&gltf_handle.0)
        && let Some(state) = asset_server.get_load_state(&gltf_handle.0)
        && state.is_loaded()
    {
        info!("GLTF loaded successfully: {:?}", gltf);
        info!("Spawn GLTF scene: {:?}", gltf.scenes[0]);
        commands.spawn((
            WorldAssetRoot(gltf.scenes[0].clone()),
            Transform::from_xyz(0., 0., 0.),
            GltfSpawned,
        ));
    } else {
        info!("GLTF not loaded yet");
        info!(
            " - Load state: {:?}",
            asset_server.get_load_state(&gltf_handle.0)
        );
    }
}
