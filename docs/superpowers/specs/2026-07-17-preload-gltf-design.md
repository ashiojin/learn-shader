# Design: Pre-load GLTF assets for the sandbox

Date: 2026-07-17
Crate: `sandbox`

## Goal

Pre-load every GLTF asset used by the sandbox at application startup, keep strong
handles to them for the entire lifetime of the process, and gate sample spawning
until loading completes. This achieves three things the user asked for:

- **Prevent unload/GC** — strong `Handle<Gltf>` values are held in a long-lived
  resource, so the assets (and their sub-assets) are never dropped while the app
  runs.
- **Warm-up at startup** — loading is kicked off in a `Startup` system rather than
  lazily on first sample selection.
- **Avoid reload hitch** — because the assets stay resident, switching to a GLTF
  sample (e.g. `Saru`, `ArmAndRod`) reuses the already-loaded data with no reload
  stutter.

## Current behavior (for reference)

- `sample/spawner.rs::spawn_sample` hardcodes GLTF paths inline (`"models/saru.glb"`,
  `"models/poc_arm_and_rod_ex.glb"`) when spawning a `SingleGltfEmitter`.
- `sample/emitter.rs::spawn_single_gltf_scene` calls `asset_server.load(...)` each
  time a GLTF sample is selected. The spawned scene entity holds the only strong
  handle; switching away despawns it, dropping the handle. The asset may then be
  unloaded, forcing a reload (and a hitch) next time.
- `SampleState::default()` is inserted at plugin build; `refresh_sample_mesh` runs
  under `run_if(resource_changed::<SampleState>)`, which fires on the first frame to
  spawn the default sample.

## Design

### Overview

Introduce a `PreloadPlugin` (a new self-contained feature plugin, per the repo
convention in CLAUDE.md) that owns:

1. An `AppState` (Bevy `States`) with `Loading` and `Running` variants.
2. A `PreloadedGltf` resource holding the retained `Handle<Gltf>` values.
3. A startup loader and a load-completion poller that drives the state transition.

`SampleType` gains a method that exposes the GLTF asset descriptor for its
GLTF-backed variants, so the preload list is *derived from `SampleType`* and stays
in sync with the available samples automatically.

### Unit 1 — `SampleType::gltf_asset` (in `sample/state.rs`)

A small, pure descriptor plus an accessor:

```rust
#[derive(Debug, Clone, Copy)]
pub struct GltfSampleAsset {
    pub path: &'static str,
    pub scene_idx: usize,
}

impl SampleType {
    /// Returns the GLTF asset descriptor for GLTF-backed variants, else None.
    pub fn gltf_asset(&self) -> Option<GltfSampleAsset> {
        match self {
            Self::Saru => Some(GltfSampleAsset { path: "models/saru.glb", scene_idx: 0 }),
            Self::ArmAndRod => Some(GltfSampleAsset {
                path: "models/poc_arm_and_rod_ex.glb", scene_idx: 0,
            }),
            _ => None,
        }
    }
}
```

- **What it does:** single source of truth for which `SampleType`s are GLTF-backed
  and where their files live.
- **How it's used:** by the preloader (iterate `all_variants()`, collect
  `gltf_asset()`) and by `spawn_sample`.
- **Depends on:** nothing beyond `SampleType`.

`spawn_sample` in `spawner.rs` is refactored so the `Saru`/`ArmAndRod` arms build
their `SingleGltfEmitter` from `gltf_asset()` instead of inline string literals,
removing the duplicated paths. (Animation config — `AutoAnimation` — stays where it
is; it is a spawn concern, not part of the asset descriptor.)

### Unit 2 — `AppState` + `PreloadedGltf` (new module `sample/preload.rs`)

```rust
#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AppState {
    #[default]
    Loading,
    Running,
}

#[derive(Resource, Default)]
pub struct PreloadedGltf {
    pub handles: Vec<Handle<Gltf>>, // strong handles, kept for the app lifetime
}
```

- **What it does:** `AppState` tracks whether preloading is done; `PreloadedGltf`
  retains the strong handles so the assets are never unloaded.
- **Rationale for `Handle<Gltf>` (Axis 1, option A):** holding the top-level `Gltf`
  handle keeps all its sub-assets (scenes, meshes, animation clips) resident, so the
  existing `asset_server.load(GltfAssetLabel::Scene/Animation …)` calls in
  `spawn_single_gltf_scene` resolve to cached handles with no reload. Spawn code is
  left essentially unchanged.

### Unit 3 — Systems (in `sample/preload.rs`)

**`preload_gltf_assets`** — `Startup`:

```rust
fn preload_gltf_assets(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let handles = SampleType::all_variants()
        .iter()
        .filter_map(|s| s.gltf_asset())
        .map(|a| a.path)
        .collect::<std::collections::BTreeSet<_>>() // dedup by path
        .into_iter()
        .map(|path| asset_server.load::<Gltf>(path.to_string()))
        .collect();
    commands.insert_resource(PreloadedGltf { handles });
}
```

Loads each distinct GLTF file as a whole `Gltf` asset and stores the strong handles.
De-duplicates by path so two `SampleType`s sharing a file load once.

**`check_preload_complete`** — `Update`, `run_if(in_state(AppState::Loading))`:

```rust
fn check_preload_complete(
    asset_server: Res<AssetServer>,
    preloaded: Res<PreloadedGltf>,
    mut next: ResMut<NextState<AppState>>,
) {
    use bevy::asset::RecursiveDependencyLoadState::*;
    let all_settled = preloaded.handles.iter().all(|h| {
        matches!(
            asset_server.recursive_dependency_load_state(h),
            Loaded | Failed(_)
        )
    });
    if all_settled {
        next.set(AppState::Running);
    }
}
```

Polls recursive dependency load state for every retained handle. Transitions to
`Running` once all are `Loaded` (or `Failed`, so a bad asset can't hang the app —
the failure is logged and the app proceeds). `PreloadedGltf` may be absent for the
first frame or two before `preload_gltf_assets`' `insert_resource` is applied; the
run condition is additionally guarded with `resource_exists::<PreloadedGltf>`.

**`trigger_initial_sample`** — `OnEnter(AppState::Running)`:

```rust
fn trigger_initial_sample(mut sample_state: ResMut<SampleState>) {
    sample_state.set_changed(); // force refresh_sample_mesh to spawn the default sample
}
```

Guarantees the default sample spawns exactly when the app becomes `Running`,
independent of first-frame change-detection timing.

### Unit 4 — Wiring

Ownership split keeps loading mechanics separate from sample logic:

- **`PreloadPlugin` (`sample/preload.rs`)** owns the loading machinery. Its
  `build()` calls `app.init_state::<AppState>()`, registers `preload_gltf_assets`
  on `Startup`, and registers `check_preload_complete` on `Update` with
  `run_if(in_state(AppState::Loading).and(resource_exists::<PreloadedGltf>))`. It is
  added from `main.rs` alongside the other feature plugins (matching the repo's
  one-plugin-per-feature convention).
- **`SamplePlugin`** consumes `AppState`: `refresh_sample_mesh` gains
  `.run_if(in_state(AppState::Running))` in addition to its existing
  `resource_changed::<SampleState>` condition, and `trigger_initial_sample` is
  registered on `OnEnter(AppState::Running)` (it lives here because it touches
  `SampleState`, which `SamplePlugin` owns).
- The downstream spawn systems (`spawn_single_gltf_scene`, `spawn_single_mesh`, …)
  are left as-is: they react to `Added<…Emitter>` components, and no emitters exist
  until `refresh_sample_mesh` runs in `Running`, so gating `refresh_sample_mesh`
  alone is sufficient.

`AppState` is defined in `sample/preload.rs` and re-exported so `SamplePlugin` can
reference it; `preload.rs` depends on `super::state::SampleType`. This keeps the
mutual reference inside the `sample` module (no top-level module cycle).

### Data flow

```
Startup:  preload_gltf_assets --> PreloadedGltf { handles: [Handle<Gltf>, ...] }
Update (Loading): check_preload_complete --> (all settled) --> NextState(Running)
OnEnter(Running): trigger_initial_sample --> SampleState.set_changed()
Update (Running): refresh_sample_mesh (resource_changed) --> spawn_sample
                    --> SingleGltfEmitter --> spawn_single_gltf_scene
                    --> asset_server.load(Scene label) resolves to cached handle
```

The `PreloadedGltf` resource is never removed, so the handles live for the whole run.

## Error handling

- A GLTF that fails to load counts as "settled" (`Failed`) so it cannot deadlock the
  loading gate. `check_preload_complete` logs a `warn!` for any handle in `Failed`
  state before transitioning.
- Acquiring `PreloadedGltf` before it is inserted is avoided via a
  `resource_exists` run condition on `check_preload_complete`.

## Testing

- **Unit test (in `sample/state.rs`):** assert every path returned by
  `SampleType::all_variants().gltf_asset()` is non-empty and that the known
  GLTF-backed variants (`Saru`, `ArmAndRod`) return `Some`, non-GLTF variants return
  `None`. This is the pure, deterministic piece worth a test.
- **Manual / runtime verification:** launch the sandbox; confirm (a) the app starts
  in `Loading` and transitions to `Running`, (b) the default Saru sample appears,
  (c) cycling to `ArmAndRod` and back is hitch-free, (d) the GLTF assets remain
  loaded after switching to a non-GLTF sample and back. Bevy asset I/O and state
  transitions are not practically unit-testable here, so these are checked at
  runtime.

## Cross-platform notes

- Deriving the list from `SampleType` needs no filesystem scan, so this works
  identically on native and WASM (`asset_server.load` and Bevy `States` are
  available on both). No native-only code is introduced.

## Out of scope (YAGNI)

- Exposing load progress through the HTTP API / telemetry (`AppStatus`).
- A visible on-screen loading UI/spinner (the gate is logic-only; the window simply
  shows the background until `Running`).
- Preloading non-GLTF assets (textures already load via `asset_server` where used).
