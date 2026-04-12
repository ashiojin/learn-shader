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


@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    var color = vec3(0.0);
    //color = uneune(in.uv, globals.time);
    // let color2 = block_noise(in.uv);
    //color = mix(color, color2, smoothstep(0.49, 0.51, (sin(globals.time * 2.0) + 1.0) / 2.));
    //color = color;

    //let n = 0.0;
    let n = snoise3D(vec3(in.uv * 5.0, globals.time * 0.5)) * 0.5 + 0.5;
    //let p = rotate2d(n * sin(globals.time * 2.3) * 0.5 * PI) * ((in.uv - 0.5) * 2.);
    //let p = scale2d(abs(n)) * ((in.uv - 0.5) * 2.);
    let p = scale2d(n) * rotate2d(n * sin(globals.time) * 2. * PI) * ((in.uv - 0.5) * 2.);
    let l = horizontal_lines(p, 1.) + vertical_lines(p, 1.);
    color = vec3(l, n, p.x *p.x + p.y * p.y);

    //let color = truche(in.uv);
    return vec4<f32>(color, 1.0);
}
