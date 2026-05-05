# Sandbox

A Sanxbox to write a fragment shader.

## Usage

This app uses the fragment shaders:

- `assets/shaders/fragment.wgsl`
- `assets/shaders/extended_material.wgsl`

A mesh is displayed to which the fragment shader is applied.

- wasd: Rotate the mesh
- q: Reset the camera
- n: Change the mesh
- r: Reload shaders
- 0: Show a cross at the origin
- b: Change the background
- l: Change the light
- 1: Change the material (fragment.wgsl <-> uv test texture)

## TODO

- Allow the window to be resized
- Add some inputs to the fragment shader
  - float values
  - textures
- More background
  - Some objects behind sample
  - Some objects around sample
- File selector to read other fragment shaders
- More meshes?

## Issues

- On rendering a spherical ring's meshes, alpha blending is broken. The meshes would not be sorted. 
  - Check `CustomMaterial` and `ExtendedMaterial` are treated as `SortedPhaseItem`.
