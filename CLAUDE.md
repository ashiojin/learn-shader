# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

Learning notes for Bevy shader programming (Bevy `0.19`). A Cargo workspace (`resolver = "3"`, edition 2024) of several small crates, each transcribing or extending an official Bevy shader example.

## Workspace layout

- `sandbox/` — the main, actively-developed app. A live shader "sandbox": one mesh on screen with a switchable material/shader, driven by keyboard and an HTTP API. Depends on `myshaderlib` and `my_meshes`.
- `myshaderlib/` — shared WGSL library (`lib.wgsl`, `simplex_noise.wgsl`) plus a UV-test texture, all shipped as **embedded assets** via `MyShaderLibPlugin` (loaded from `embedded://myshaderlib/...`).
- `my_meshes/` — custom procedural `Meshable` types (`FlatRing3d`, `SphericalZone`, `Belt`, `Trail`) and the custom vertex attribute `ATTRIBUTE_TIME`.
- `ex-1-generate-pattern/`, `ex-2-custom-render-phase/` — standalone example binaries with their own `assets/`.

## Build & run

Cross-compilation is central here: development targets **Windows MSVC from Linux/WSL** (see `GEMINI.md`, `.vscode/settings.json`, `.cargo/config.toml` which sets `linker = "lld-link"`).

```bash
# Windows (MSVC) — requires xwin; see below
cargo build --target x86_64-pc-windows-msvc
cargo build -p sandbox --target x86_64-pc-windows-msvc

# Run a single example binary
cargo run -p ex-1-generate-pattern --target x86_64-pc-windows-msvc

# Tests (unit tests live in my_meshes/ and myshaderlib/)
cargo test -p my_meshes
cargo test -p my_meshes belt      # single test by name filter
```

The MSVC `build.rs` (present in every crate) **panics** unless `XWIN_CACHE_DIR` points at an `xwin splat` output. Setup: `cargo install xwin` → `xwin --accept-license splat --output <dir>` → `export XWIN_CACHE_DIR=<absolute dir>`. This logic only fires for `msvc` targets; it is a no-op otherwise.

### WASM build (sandbox)

The HTTP API is native-only; WASM exposes a `start()` entry point (`#[wasm_bindgen]`) instead. Build steps live in `sandbox/README.md`:

```bash
cargo build --release --target wasm32-unknown-unknown -p sandbox
wasm-bindgen --no-typescript --target web --out-dir ./sandbox/out-web/ target/wasm32-unknown-unknown/release/sandbox.wasm
# copy src-web/index.html into out-web/, then serve out-web/ over http
```

The `sandbox` `Cargo.toml` contains **two required workarounds** (do not remove without cause): `blake3` with `features=["pure"]` (bevy issue #10425, in every crate), and a `getrandom` `wasm_js` feature pin for the wasm target.

## sandbox architecture

`main.rs` assembles the app from feature plugins: `MyShaderLibPlugin`, `RandomPlugin`, `SamplePlugin`, `SatelliteCameraPlugin`, `BackgroundPlugin`, `LightPlugin`, `DebugGizmoPlugin`, `BillboardPlugin`, `UnifiedApiPlugin`. Each self-contained feature is its own plugin — follow that pattern when adding features (`sandbox/TODO.md` lists "separate each feature into libs" as a standing goal).

**`SamplePlugin` (`src/sample/`) is the core.** It manages a `SampleState` resource (current `SampleType` mesh + `SampleMaterialType` material + billboard flag). Changing state (via keys or API) re-runs `refresh_sample_mesh` (`run_if(resource_changed::<SampleState>)`), which respawns the displayed mesh/scene and reapplies the material. Both mesh switching (`SampleType`) and material switching (`SampleMaterialType`) are cyclic enums with `all_variants` / `get_next` / `as_str` / `from_str` — add new samples/materials by extending both the enum and its `all_variants`/`as_str` arms.

**Shader hot-reload is the defining mechanism.** The two in-memory shaders (`CustomMaterial` ← `src/shaders/fragment.wgsl`, `ExtendedMaterial` ← `src/shaders/extended_material.wgsl`) are `include_str!`'d into `OnceLock<RwLock<String>>` statics in `api_shared.rs`, mirrored into `CustomMaterialShader` / `ExtendedMaterialShader` resources. On a `ReloadReq` message, `load_custom_material` / `load_global_res` rebuild a `Shader` from the current string and `Assets<Shader>::insert` it at a **fixed `Uuid`** (`CUSTOM_MATERIAL_WGSL_UUID`, `EXTENDED_MATERIAL_WGSL_UUID`); the materials reference those UUIDs via `ShaderRef::Handle`. So editing shader text and firing `ReloadReq` swaps the running shader without touching the asset filesystem.

**HTTP API (`src/api/`, native only).** `UnifiedApiPlugin` spawns an axum server (default port `3000`, `API_PORT` override, CORS-open) on a background thread. `GET/POST /wgsl/{CustomMaterial|ExtendedMaterial}` read/write the shader strings; `POST /command` and `GET /status` drive/inspect state. Cross-thread handoff is a `Mutex<Vec<ApiCommand>>` queue drained each frame by `poll_api_commands`; `sync_telemetry_cache` publishes `AppStatus` back out. WASM uses `src/api/wasm.rs` exported functions instead.

**Custom vertex attributes & `specialize`.** Both `CustomMaterial` and the `ExtendedMaterial` extension override `specialize()` to manually rebuild the vertex buffer layout, because meshes may carry `my_meshes::ATTRIBUTE_TIME` (spawn time, shader location 10). When present, the code pushes the `MY_MESHES_ATTRIBUTE_TIME` shader-def to vertex + fragment stages. The two `specialize` impls are near-duplicates (noted `FIXME` in `material.rs`) — keep them in sync. Note the base attribute location tables **differ between the main pass and the prepass** (`is_prepass_pipeline` branch); preserve that distinction.

**WebGL2 alignment caveat.** Uniform structs (`CustomMaterial`, `MyExtension`) carry explicit `_weggl2_padding_*` fields so the uniform buffer is 16-byte aligned — required by the default `webgl2` bevy feature. Keep padding when adding uniform fields.

**Emitter/trail systems.** `sample/emitter.rs` + `scene_mod.rs` implement particle-like emitters and trails. Trails need up-to-date `GlobalTransform`, so `spawn_trail_from_emitter` runs in `PostUpdate` `.after(TransformSystems::Propagate)` (see commits about resolving a 1-frame trail delay).

## Conventions

- Assets are large binaries tracked with **Git LFS** (`.gitattributes`: `*.glb *.png *.mp4 *.ttf *.ogg`).
- `sandbox` reads its asset root from the `ASSETS_DIR` env var (defaults to `assets`).
- Bevy is pulled in per-crate with a curated `features` list; the workspace pins `bevy = { version = "0.19", default-features = false }`. Match the existing feature set when adding a crate.
- Comments and commit-adjacent notes are frequently in Japanese; both languages appear throughout.
