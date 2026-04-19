#import bevy_pbr::{
    mesh_view_bindings::globals,
    forward_io::VertexOutput,
}

#import mylib::{
    rand, rand2, noise,
    PI,
    rotate2d, scale2d,
    horizontal_lines, vertical_lines,
}

#import simplex_noise_f32::{snoise3D}

fn block_noise(uv: vec2<f32>) -> vec3<f32> {
    let st = uv * 16.0;

    let ipos = floor(st);
    let fpos = fract(st);

    let color = vec3(rand2(ipos));
    return color;
}
fn uneune(uv: vec2<f32>, time: f32) -> vec3<f32> {
    let st = uv * 16.0;
    let ipos = floor(st);
    let fpos = fract(st);

    let a = rand2(ipos);
    let b = rand2(ipos + vec2(1.0, 0.0));
    let c = rand2(ipos + vec2(0.0, 1.0));
    let d = rand2(ipos + vec2(1.0, 1.0));

    let f = fpos * fpos * (3.0 - 2.0 * fpos);
    //let f = fpos;

    let v = mix(a, b, f.x) +
            (c - a) * f.y * (1.0 - f.x) +
            (d - b) * f.x * f.y;
    let sint = noise(time);
    let k = 0.4;
    let h = ((sin(time) + 1.0) / 2.) * 0.1 + 0.01;
    let t = sint * k + (1.0 - k) /2.;
    let line = step(t - h, v) - step(t + h, v);
    return vec3(line);
}
fn truche_tile(uv: vec2<f32>, index: f32) -> vec2<f32> {
    let _index = fract((index-0.5) * 2.0);
    if (_index > 0.75) {
        return vec2(1.0) - uv;
    } else if (_index > 0.5) {
        return vec2(1.0-uv.x, uv.y);
    } else if (_index > 0.25) {
        return 1.0 - vec2(1.0 - uv.x, uv.y);
    } else {
        return uv;
    }
}
fn truche(uv: vec2<f32>) -> vec3<f32> {
    let st = uv * 8.0;

    let ipos = floor(st);
    let fpos = fract(st);

    let tile = truche_tile(fpos, rand2(ipos));
    // let color = smoothstep(tile.x - 0.3, tile.x, tile.y) -
    //     smoothstep(tile.x, tile.x + 0.3, tile.y);
    //let color = vec3<f32>(tile.x, tile.y, 1.0);
    let color = step(tile.x, tile.y);
    return vec3(color);
}

fn rect(uv:vec2<f32>, p: vec2<f32>, wh: vec2<f32>) -> f32 {

    // 1.0: inside rectangle(p -> (p + wh))
    // 0.0: outside rectangle
    return step(p.x, uv.x) * step(uv.x, p.x + wh.x) *
           step(p.y, uv.y) * step(uv.y, p.y + wh.y);
}


@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    var color = vec4(0.0);
    //color = uneune(in.uv, globals.time);
    // let color2 = block_noise(in.uv);
    //color = mix(color, color2, smoothstep(0.49, 0.51, (sin(globals.time * 2.0) + 1.0) / 2.));
    //color = color;

    //let n = 0.0;
    let o = in.uv;
    let t = fract(globals.time);
    let p = vec2(o.x + 0.01*sin(o.y * 2.*PI), pow(o.y, 0.42));
    let n_move =  vec2(0.0, globals.time * 5.5);
    let n = snoise3D(vec3(p * 8.0 - n_move, globals.time * 2.5)) * 0.5 + 0.5;
    //let p = rotate2d(n * sin(globals.time * 2.3) * 0.5 * PI) * ((in.uv - 0.5) * 2.);
    //let p = scale2d(abs(n)) * ((in.uv - 0.5) * 2.);
    //let p = scale2d(n) * rotate2d(n * sin(globals.time) * 2. * PI) * ((in.uv - 0.5) * 2.);
    let pp = vec2(n);
    let d = p.y;// * p.y;

    let color_base = vec4(1.0, 0.0, 0.0, 1.0);
    let color_half = vec4(1.0, 0.8, 0.1, 1.0);
    let color_end  = vec4(1.0, 1.0, 0.5, 0.0);
    let th_half = 0.5;
    color = mix(color_base, color_half, smoothstep(0.0, th_half, d + n * 0.1));
    color = mix(color, color_end, smoothstep(th_half, 1.0, d + n * 0.1));
    // if d < th_half {
    //     color = mix(color_base, color_half, smoothstep(0.0, th_half, d + n * 0.1));
    // } else {
    //     color = mix(color_half, color_end, smoothstep(th_half, 1.0, d + n * 0.1));
    // }
    //color = vec3(l, n, p.x *p.x + p.y * p.y);
    // if abs(in.uv.x * in.uv.y) < 0.001 {
    //     color = vec3(1.);
    // } else if abs(in.uv.x * in.uv.y) > 1.0 {
    //     color = vec3(0.);
    // }

    //let color = truche(in.uv);
    return vec4<f32>(color);
}
