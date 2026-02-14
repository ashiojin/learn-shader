#import bevy_pbr::{
    pbr_fragment::pbr_input_from_standard_material,
    pbr_functions::alpha_discard,

    pbr_types::PbrInput,
}

#ifdef PREPASS_PIPELINE
#import bevy_pbr:: {
    prepass_io::{VertexOutput, FragmentOutput},
    pbr_deferred_functions::deferred_output,
}
#else
#import bevy_pbr:: {
    forward_io::{VertexOutput, FragmentOutput},
    pbr_functions::{apply_pbr_lighting, main_pass_post_lighting_processing},
}
#endif

struct MyMaterial {
    x: u32,
#ifdef SIXTEEN_BYTE_ALIGNMENT
    _webgl2_padding_8b: u32,
    _webgl2_padding_12b: u32,
    _webgl2_padding_16b: u32,
#endif
}

struct Pattern {
    base_color: vec4<f32>,
    metallic: f32,
}
fn make_pattern(
    x: u32,
    color_1: vec4<f32>,
    color_2: vec4<f32>,
    metallic_1: f32,
    metallic_2: f32,
    uv: vec2<f32>,
) -> Pattern {

    // checker
    // +----+----+
    // | 0  | .5 |
    // +----+----+
    // | .5 | 1  |
    // +----+----+
    let checker_ = step(vec2<f32>(0.5), fract(uv * f32(x))) * 0.5;
    let checker = checker_.x + checker_.y;

    // checker_x2 is each cell is divided into 4 sub-cells of `checkr`
    // +----+----+
    // | 0  | 0  |
    // +----+----+
    // | 0  | 1  |
    // +----+----+
    let checker_x2_ = step(vec2<f32>(0.5), fract(uv * f32(x) * 2.0));
    let checker_x2 = checker_x2_.x * checker_x2_.y;

    let base_color = mix(color_1, color_2, checker);
    let metallic = mix(metallic_1, metallic_2, checker_x2);

    var pattern = Pattern(base_color, metallic);
    return pattern;
}

@group(#{MATERIAL_BIND_GROUP}) @binding(100)
var<uniform> my_material: MyMaterial;

@fragment
fn fragment(
    in: VertexOutput,
    @builtin(front_facing) is_front: bool,
) -> FragmentOutput {
    var pbr_input = pbr_input_from_standard_material(in, is_front);

    let uv = in.uv; // If `VERTEX_UVS_A` is't defined, it causes a compile error!
    // https://github.com/bevyengine/bevy/blob/e696fa75260d52129df53db4328aca64c068d613/crates/bevy_pbr/src/render/forward_io.wgsl#L38
    let pattern = make_pattern(my_material.x,
        pbr_input.material.base_color,
        vec4<f32>(0.5, 0.5, 0.5, 1.0),
        pbr_input.material.metallic,
        1.0,
        uv);

    // Mofiy the PBR input based on the pattern
    pbr_input.material.base_color = pattern.base_color;
    pbr_input.material.metallic = pattern.metallic;

    // alpha discard
    pbr_input.material.base_color = alpha_discard(pbr_input.material, pbr_input.material.base_color);

#ifdef PREPASS_PIPELINE // -------------
    let out = deferred_output(in, pbr_input);
#else // -------------------------------
    var out: FragmentOutput;
    out.color = apply_pbr_lighting(pbr_input);

    // Apply in-shader post processing
    out.color = main_pass_post_lighting_processing(pbr_input, out.color);

#endif // ------------------------------

    return out;
}

