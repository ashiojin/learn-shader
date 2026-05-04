#import bevy_pbr::{
    mesh_view_bindings::globals,
    pbr_fragment::pbr_input_from_standard_material,
    pbr_functions::alpha_discard,
}

#import bevy_pbr::{
    forward_io::{VertexOutput, FragmentOutput},
    pbr_functions::{apply_pbr_lighting, main_pass_post_lighting_processing},
}

struct MyExtendedMaterial {
    param1: vec4<f32>,
}

@group(#{MATERIAL_BIND_GROUP}) @binding(128)
var<uniform> my_extended_material: MyExtendedMaterial;

@fragment
fn fragment(
    in: VertexOutput,
    @builtin(front_facing) is_front: bool,
) -> FragmentOutput {
    var pbr_input = pbr_input_from_standard_material(in, is_front);

    pbr_input.material.base_color = alpha_discard(pbr_input.material, pbr_input.material.base_color);

    var out: FragmentOutput;
    out.color = apply_pbr_lighting(pbr_input);

    out.color = main_pass_post_lighting_processing(pbr_input, out.color);

    let t_sin = sin(globals.time * 2.);

    out.color = mix(out.color, my_extended_material.param1, (t_sin + 1.) / 2.);

    return out;
}

