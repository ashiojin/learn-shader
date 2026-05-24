# Sandbox

A Sanxbox to write a fragment shader.

## Usage

This app shows a mesh with a shader:

- `assets/shaders/fragment.wgsl`
  - fragment shader only
- `assets/shaders/extended_material.wgsl`
  - vertex shader and fragment shader

A mesh is displayed to which the fragment shader is applied.

- wasd: Rotate the mesh
- q: Reset the camera
- n: Change the mesh
- r: Reload shaders
- 0: Show a cross at the origin
- b: Change the background
- l: Change the light
- 1: Change the material
  - fragment.wgsl
  - UV texture
  - extended_material.wgsl

## TODO

- ALWAYS:
  - Separate each features to libs
- Using Gltf extensions instead of extras to replace a material to a extended material
- Add some inputs to the fragment shader
  - float values
  - textures
- More background
  - Some objects behind sample
  - Some objects around sample
- File selector to read other fragment shaders
- More models?


## Issues


