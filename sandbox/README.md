# Sandbox

A Sandbox to write and live-edit shaders online.

## Usage

A mesh is displayed to which the active shader is applied. Shaders are loaded into memory on startup from initial files under `src/shaders/`, and can be read and live-edited online via the HTTP API.

- wasd: Rotate the mesh
- q: Reset the camera
- n: Change the mesh
- r: Reload shaders from files (for assets loaded via AssetServer)
- 0: Show a cross at the origin
- b: Change the background
- l: Change the light
- 1: Change the material
  - CustomMaterial (in-memory, editable via API)
  - UV texture
  - ExtendedMaterial (in-memory, editable via API)
- 2: Toggle billboard mode

## Online Read/Write API Server

The sandbox runs an HTTP API server on a background thread to enable external text editors or browser-based tools to dynamically read and write the WGSL shader sources online. The API operates in-memory and triggers live hot-reloading in Bevy immediately upon update.

### Configuration
*   **Port**: `3000` (default), overridable via the `API_PORT` environment variable.
*   **CORS**: Enabled (Cross-Origin Resource Sharing is allowed for any origin to ease browser integration).

### API Endpoints
*   **Read WGSL Shader Source**:
    *   `GET /wgsl/CustomMaterial` — Reads the current CustomMaterial shader source.
    *   `GET /wgsl/ExtendedMaterial` — Reads the current ExtendedMaterial shader source.
*   **Write/Update WGSL Shader Source**:
    *   `POST /wgsl/CustomMaterial` — Accepts raw WGSL as text to update the CustomMaterial shader and triggers a reload.
    *   `POST /wgsl/ExtendedMaterial` — Accepts raw WGSL as text to update the ExtendedMaterial shader and triggers a reload.

### Examples
*   **Read a shader using curl**:
    ```bash
    curl http://localhost:3000/wgsl/CustomMaterial
    ```
*   **Upload a shader update using curl**:
    ```bash
    curl -X POST -d "@src/shaders/fragment.wgsl" http://localhost:3000/wgsl/CustomMaterial
    ```

## BUILD

### For Windows(MSVC)

```bash
cargo build --target x86_64-pc-windows-msvc
```

### For WASM

Web APIs are not supported for WASM version.
Instead of Web APIs, WASM APIs (functions) are supported. See src/wasm_api.rs

```bash
cargo install wasm-bindgen-cli
```

```bash
cargo build --release --target wasm32-unknown-unknown
wasm-bindgen --no-typescript --target web --out-dir ./out-web/ ../target/wasm32-unknown-unknown/release/sandbox.wasm
cp src-web/index.html out-web/

# Run
cd out-web/
python3 -m http.server 8080
```

## TODO

- ALWAYS:
  - Separate each features to libs
- WASM version
  - New index.html
  - new WASM Api reading status
- Add some inputs to the fragment shader
  - float values
  - textures
- More background
  - Some objects behind sample
  - Some objects around sample
- File selector to read other fragment shaders
- More models?


## Issues


