# Pre-load GLTF Assets Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Pre-load every GLTF asset the sandbox uses at startup, keep strong handles for the app's lifetime, and gate sample spawning behind a loading state so switching to a GLTF sample never reloads.

**Architecture:** A new `PreloadPlugin` (`sandbox/src/sample/preload.rs`) owns a Bevy `AppState { Loading, Running }` and a `PreloadedGltf` resource holding retained `Handle<Gltf>` values. A `Startup` system loads each distinct GLTF file (paths derived from `SampleType`); an `Update`-in-`Loading` poller flips to `Running` once all handles settle. `SamplePlugin` gates `refresh_sample_mesh` on `Running` and triggers the first sample on `OnEnter(Running)`.

**Tech Stack:** Rust (edition 2024), Bevy `0.19` (features include `3d` → `bevy_state`, so `States`/`init_state`/`OnEnter`/`in_state` are available and `StatesPlugin` ships in `DefaultPlugins`), `bevy::gltf::Gltf`.

## Global Constraints

- Bevy pinned at `0.19`, `default-features = false`; `sandbox` uses `features = ["3d"]`. Do not add bevy features — `bevy_state` is already pulled in transitively by `3d`.
- Keep the code cross-platform (native + WASM). Do NOT introduce filesystem scanning; the preload list is derived from `SampleType`, which compiles on both targets.
- Comments may be English or Japanese (repo convention); either is acceptable.
- The repo's primary compile target is `x86_64-pc-windows-msvc` (requires `XWIN_CACHE_DIR`). Pure unit tests run on the host target instead.
- Follow the one-plugin-per-feature convention used across `sandbox` (`BackgroundPlugin`, `LightPlugin`, etc.).

---

### Task 1: `SampleType::gltf_asset` descriptor + refactor `spawn_sample`

Introduce a single source of truth for which `SampleType` variants are GLTF-backed and where their files live, then make `spawn_sample` consume it (removing the inline path literals).

**Files:**
- Modify: `sandbox/src/sample/state.rs` (add `GltfSampleAsset` struct, `SampleType::gltf_asset`, and a `#[cfg(test)]` module)
- Modify: `sandbox/src/sample/spawner.rs:15-37` (the `Saru` and `ArmAndRod` arms of `spawn_sample`)

**Interfaces:**
- Consumes: `SampleType` (existing enum in `state.rs`), `SampleType::all_variants()`.
- Produces:
  - `pub struct GltfSampleAsset { pub path: &'static str, pub scene_idx: usize }` (derives `Debug, Clone, Copy`).
  - `impl SampleType { pub fn gltf_asset(&self) -> Option<GltfSampleAsset> }` — `Some` for `Saru`/`ArmAndRod`, `None` otherwise. Task 2 relies on this signature.

- [ ] **Step 1: Write the failing test**

Append to the end of `sandbox/src/sample/state.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gltf_asset_matches_known_variants() {
        // GLTF-backed variants expose a non-empty path.
        let saru = SampleType::Saru.gltf_asset().expect("Saru is GLTF-backed");
        assert_eq!(saru.path, "models/saru.glb");
        assert_eq!(saru.scene_idx, 0);

        let arm = SampleType::ArmAndRod
            .gltf_asset()
            .expect("ArmAndRod is GLTF-backed");
        assert_eq!(arm.path, "models/poc_arm_and_rod_ex.glb");
        assert_eq!(arm.scene_idx, 0);

        // Non-GLTF variants return None.
        assert!(SampleType::Cube.gltf_asset().is_none());
        assert!(SampleType::Plane.gltf_asset().is_none());
        assert!(SampleType::Emitter1.gltf_asset().is_none());
    }

    #[test]
    fn every_gltf_asset_path_is_non_empty() {
        for variant in SampleType::all_variants() {
            if let Some(asset) = variant.gltf_asset() {
                assert!(
                    !asset.path.is_empty(),
                    "{:?} has an empty GLTF path",
                    variant
                );
            }
        }
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p sandbox gltf_asset`
Expected: FAIL — compile error, `no method named gltf_asset found for enum SampleType` (and `GltfSampleAsset` not found).

- [ ] **Step 3: Add `GltfSampleAsset` and `SampleType::gltf_asset`**

In `sandbox/src/sample/state.rs`, add the struct just below the `use bevy::prelude::*;` line (after line 1):

```rust
/// Describes a GLTF asset backing a [`SampleType`] variant.
/// Single source of truth for the sandbox's GLTF file paths, consumed by both
/// the preloader and `spawn_sample`.
#[derive(Debug, Clone, Copy)]
pub struct GltfSampleAsset {
    pub path: &'static str,
    pub scene_idx: usize,
}
```

Then add this method inside the existing `impl SampleType { ... }` block (e.g. after `get_next`):

```rust
    /// Returns the GLTF asset descriptor for GLTF-backed variants, else `None`.
    pub fn gltf_asset(&self) -> Option<GltfSampleAsset> {
        match self {
            Self::Saru => Some(GltfSampleAsset {
                path: "models/saru.glb",
                scene_idx: 0,
            }),
            Self::ArmAndRod => Some(GltfSampleAsset {
                path: "models/poc_arm_and_rod_ex.glb",
                scene_idx: 0,
            }),
            _ => None,
        }
    }
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p sandbox gltf_asset`
Expected: PASS — `gltf_asset_matches_known_variants` and `every_gltf_asset_path_is_non_empty` both pass.

- [ ] **Step 5: Refactor `spawn_sample` to consume `gltf_asset`**

In `sandbox/src/sample/spawner.rs`, replace the `Saru` arm (lines 17-26) with:

```rust
        SampleType::Saru => {
            let asset = sample_type.gltf_asset().expect("Saru is GLTF-backed");
            commands.spawn((
                SingleGltfEmitter {
                    gltf_path: asset.path.to_string(),
                    scene_idx: asset.scene_idx,
                },
                Transform::from_xyz(0., 0., 0.),
                SampleEmitter,
            ));
        }
```

And replace the `ArmAndRod` arm (lines 27-37) with:

```rust
        SampleType::ArmAndRod => {
            let asset = sample_type.gltf_asset().expect("ArmAndRod is GLTF-backed");
            commands.spawn((
                SingleGltfEmitter {
                    gltf_path: asset.path.to_string(),
                    scene_idx: asset.scene_idx,
                },
                Transform::from_xyz(0., 0., 0.).with_scale(Vec3::new(0.3, 0.3, 0.3)),
                SampleEmitter,
                AutoAnimation::new(0, AnimationType::Repeat),
            ));
        }
```

Note: `GltfSampleAsset` is reachable in `spawner.rs` via the existing `use super::state::{...}` path — add `GltfSampleAsset` to that import only if the compiler reports it unresolved (it is used indirectly through the method's return type, so no import is strictly required).

- [ ] **Step 6: Verify the refactor compiles and tests still pass**

Run: `cargo test -p sandbox gltf_asset`
Expected: PASS (behavior unchanged — the spawned emitter paths are identical to before).

- [ ] **Step 7: Commit**

```bash
git add sandbox/src/sample/state.rs sandbox/src/sample/spawner.rs
git commit -m "feat: add SampleType::gltf_asset descriptor and use it in spawn_sample"
```

---

### Task 2: `PreloadPlugin` — load GLTF at startup, retain handles, gate on completion

Create the loading machinery: `AppState`, the `PreloadedGltf` resource, the startup loader, and the completion poller. Wire the module and plugin into the app. After this task the app compiles and transitions `Loading → Running`, though sample gating is added in Task 3.

**Files:**
- Create: `sandbox/src/sample/preload.rs`
- Modify: `sandbox/src/sample.rs` (declare `pub mod preload;` and re-export `AppState`, `PreloadPlugin`)
- Modify: `sandbox/src/main.rs:15-23` (import `PreloadPlugin`) and `sandbox/src/main.rs:66-77` (add it to the plugin tuple)

**Interfaces:**
- Consumes: `SampleType::all_variants()` and `SampleType::gltf_asset()` (Task 1); `bevy::gltf::Gltf`; `bevy::asset::RecursiveDependencyLoadState`.
- Produces:
  - `pub enum AppState { #[default] Loading, Running }` (derives `States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default`). Task 3 relies on `AppState::Running`.
  - `pub struct PreloadedGltf { pub handles: Vec<Handle<Gltf>> }` (derives `Resource, Default`).
  - `pub struct PreloadPlugin;` implementing `Plugin`.

- [ ] **Step 1: Create `sandbox/src/sample/preload.rs`**

```rust
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
                    in_state(AppState::Loading).and(resource_exists::<PreloadedGltf>),
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
```

- [ ] **Step 2: Declare the module and re-export in `sandbox/src/sample.rs`**

Add to the module declarations near the top of `sandbox/src/sample.rs` (alongside the other `pub mod` lines, e.g. after `pub mod material;`):

```rust
pub mod preload;
```

Add a re-export next to the existing `pub use` lines (e.g. after `pub use state::{SampleModel, SampleState};`):

```rust
pub use preload::{AppState, PreloadPlugin};
```

- [ ] **Step 3: Register `PreloadPlugin` in `sandbox/src/main.rs`**

In the `use crate::{ ... }` block (lines 15-23), change the `sample` import line:

```rust
    sample::{PreloadPlugin, SamplePlugin, SampleState},
```

In the `.add_plugins(( ... ))` tuple (lines 66-77), add `PreloadPlugin` before `SamplePlugin`:

```rust
        .add_plugins((
            default_plugin,
            myshaderlib::MyShaderLibPlugin,
            RandomPlugin,
            PreloadPlugin,
            SamplePlugin,
            camera::SatelliteCameraPlugin,
            BackgroundPlugin,
            LightPlugin,
            DebugGizmoPlugin,
            BillboardPlugin,
            UnifiedApiPlugin,
        ))
```

- [ ] **Step 4: Verify it compiles**

Run: `cargo build -p sandbox --target x86_64-pc-windows-msvc`
(Requires `XWIN_CACHE_DIR`; if cross-compiling is unavailable in this environment, run `cargo check -p sandbox` against the host target instead.)
Expected: builds successfully with no errors. There may be an unused-variable/dead-code note only if something is mis-wired — there should be none.

- [ ] **Step 5: Commit**

```bash
git add sandbox/src/sample/preload.rs sandbox/src/sample.rs sandbox/src/main.rs
git commit -m "feat: add PreloadPlugin to preload and retain GLTF assets"
```

---

### Task 3: Gate sample spawning on `AppState::Running`

Wire `SamplePlugin` to the state machine: don't spawn samples until preloading finishes, and spawn the default sample exactly when the app enters `Running`.

**Files:**
- Modify: `sandbox/src/sample.rs:20-24` (import `AppState`) and `sandbox/src/sample.rs:66-69` (gate `refresh_sample_mesh`, add `OnEnter` system + the new function)

**Interfaces:**
- Consumes: `AppState::Running` (Task 2), `SampleState` (existing), `refresh_sample_mesh` (existing).
- Produces: no new public interface. Adds a private `trigger_initial_sample` system to `sample.rs`.

- [ ] **Step 1: Make `AppState` available inside `sample.rs`**

`AppState` is defined in `sample::preload` and already re-exported at the `sample` module root (Task 2, Step 2). Inside `sample.rs` refer to it as `preload::AppState`. Confirm `use bevy::prelude::*;` is present at the top of `sample.rs` (it is — line 1) so `in_state`, `OnEnter`, and `IntoScheduleConfigs` are in scope.

- [ ] **Step 2: Gate `refresh_sample_mesh` on `Running`**

In `sandbox/src/sample.rs`, replace the existing block (lines 66-69):

```rust
        .add_systems(
            Update,
            refresh_sample_mesh.run_if(resource_changed::<SampleState>),
        )
```

with:

```rust
        .add_systems(
            Update,
            refresh_sample_mesh
                .run_if(resource_changed::<SampleState>)
                .run_if(in_state(preload::AppState::Running)),
        )
        .add_systems(
            OnEnter(preload::AppState::Running),
            trigger_initial_sample,
        )
```

- [ ] **Step 3: Add the `trigger_initial_sample` system**

Add this private function at the end of `sandbox/src/sample.rs` (after the `impl Plugin for SamplePlugin` block):

```rust
/// When preloading finishes and the app enters `Running`, force
/// `refresh_sample_mesh` to run once so the default sample is spawned. This
/// avoids relying on first-frame change-detection timing across the state
/// transition.
fn trigger_initial_sample(mut sample_state: ResMut<SampleState>) {
    sample_state.set_changed();
}
```

- [ ] **Step 4: Verify it compiles**

Run: `cargo build -p sandbox --target x86_64-pc-windows-msvc`
(Or `cargo check -p sandbox` on the host target if cross-compilation is unavailable.)
Expected: builds successfully with no errors or warnings.

- [ ] **Step 5: Manual runtime verification**

Run the sandbox (native): `cargo run -p sandbox --target x86_64-pc-windows-msvc`
Observe / confirm:
1. Startup log shows `Preloading N GLTF asset(s)` then `GLTF preload complete; entering Running state`.
2. The default **Saru** model appears on screen after loading completes.
3. Press `N` to cycle samples; switching to **ArmAndRod** and back to **Saru** is hitch-free (no reload stall) because the `Gltf` handles remain resident.
4. Switch to a non-GLTF sample (e.g. **Cube**) and back to a GLTF one — still hitch-free.

(Runtime state transitions and Bevy asset I/O are not practically unit-testable here; this manual pass is the verification for Tasks 2–3.)

- [ ] **Step 6: Commit**

```bash
git add sandbox/src/sample.rs
git commit -m "feat: gate sample spawning on AppState::Running preload completion"
```

---

## Notes for the implementer

- **Why retain `Handle<Gltf>` (not `Handle<Scene>`):** holding the top-level `Gltf` handle keeps all its sub-assets (scenes, meshes, animation clips) alive. The existing `asset_server.load(GltfAssetLabel::Scene/Animation …)` calls in `sample/emitter.rs::spawn_single_gltf_scene` then resolve to already-loaded, cached handles — no reload, and that spawn code needs no changes.
- **Change-detection across the gate:** `resource_changed::<SampleState>` compares the resource's change tick against the *system's* last-run tick. Because `refresh_sample_mesh` never runs during `Loading`, the initial `SampleState` insertion would normally still be seen on its first `Running` run — but `trigger_initial_sample` makes that guaranteed rather than timing-dependent.
- **`resource_exists::<PreloadedGltf>` guard:** `preload_gltf_assets` inserts the resource via `Commands`, which applies at the next command flush, so `check_preload_complete` must not assume the resource exists on the very first `Update`.
- Do not remove the `PreloadedGltf` resource anywhere — its lifetime *is* the retention mechanism.
