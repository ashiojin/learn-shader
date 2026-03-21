#import bevy_pbr::{
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

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let f = 30.0;
    let alpha = 0.1;
    let checkerboard = step(0.0,
        sin(in.world_position.x * f) * sin(in.world_position.y * f) * sin(in.world_position.z * f)
    );
    return vec4(1.0, 1.0, 1.0, alpha * checkerboard);
}
