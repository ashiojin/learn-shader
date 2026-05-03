# Sandbox

A Sanxbox to write a fragment shader.

## Usage

This app uses the fragment shader `assets/shaders/fragment.wgsl`.
A mesh is displayed to which the fragment shader is applied.

- wasd: Rotate the mesh
- q: Reset the camera
- n: Change the mesh
- r: Reload `assets/shaders/fragment.wgsl`
- 0: Show a cross at the origin
- b: Change the background
- l: Change the light
- 1: Change the material (fragment.wgsl <-> uv test texture)

## TODO

- Change material to [Extended material](https://bevy.org/examples-webgpu/shaders/extended-material/)
- More background
  - Some objects behind sample
  - Some objects around sample
- Add some inputs to the fragment shader
  - float values
  - textures
- File selector to read other fragment shaders
- More meshes?
