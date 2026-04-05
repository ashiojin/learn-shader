#import bevy_pbr::{
    mesh_view_bindings::globals,
    mesh_functions,
    view_transformations::position_world_to_clip
}

struct Vertex {
    @builtin(instance_index) instance_index: u32,
    @location(0) position: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec4<f32>,
}

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    var out: VertexOutput;
    var world_from_local = mesh_functions::get_world_from_local(vertex.instance_index);
    out.world_position = mesh_functions::mesh_position_local_to_world(world_from_local, vec4(vertex.position, 1.0));
    out.clip_position = position_world_to_clip(out.world_position.xyz);
    return out;
}

fn rand(x: f32) -> f32 {
    let t = fract(sin(globals.time * 11.11));
    let xx = x * 1024.0;// + globals.time * speed;
    return fract(
        fract(sin(xx * 1.72 * t))
        + fract(sin(xx * 2.31 +0.12))
        + fract(sin(t * 3245.23))
    );
}

fn noise(xy: vec2<f32>, z: f32) -> vec4<f32> {
    let base = vec3<f32>(1.0, 1.0, 1.0);
    let base_alpha = 0.5;
    var r = rand(xy.x);
    r += rand(xy.y);
    r = fract(r);
    var color = base;
    var alpha = base_alpha * pow(r, 16.0);
    return vec4<f32>(color, alpha);
}

fn checkerboard(world_xyz: vec3<f32>, w:f32) -> f32 {
    let c = step(0.0,
        sin(world_xyz.x * w) * sin(world_xyz.y * w) * sin(world_xyz.z * w)
    );
    return c;
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let ch = vec4(1.0, 1.0, 1.0, 0.1 * checkerboard(in.world_position.xyz, 30.0));
    let ns = noise(in.clip_position.xy, in.clip_position.z);
//    let sw = step(1.0, (globals.time * 0.3)% 2.0);
    var sw = (sin(globals.time * 2.0) + 0.5) * 0.5;
    sw = pow(sw, 2.0);
    return sw * ch + (1.0-sw) * ns;
}
