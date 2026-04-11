#import bevy_pbr::{
    mesh_view_bindings::globals,
    forward_io::VertexOutput,
}

fn rand(f: f32) -> f32 {
    return fract(sin(f) * 43758.5453123);
}
fn rand2(f2: vec2<f32>) -> f32 {
    return fract(sin(dot(f2.xy, vec2<f32>(12.9898, 78.233))) * 43.585453123);
    //return rand(f2.x * f2.y);
}
fn rand3(f3: vec3<f32>) -> f32 {
    return fract(sin(dot(f3, vec3<f32>(12.9898, 78.233, 45.164))) * 43.585453123);
}
fn noise(x: f32) -> f32 {
    let i = floor(x);
    let f = fract(x);
    return mix(rand(i), rand(i+1.0), smoothstep(0., 1., f));
}
fn noise2(x: vec2<f32>) -> f32 {
    let i = floor(x);
    let f = fract(x);
    let a = rand2(i);
    let b = rand2(i + vec2(1.0, 0.0));
    let c = rand2(i + vec2(0.0, 1.0));
    let d = rand2(i + vec2(1.0, 1.0));
    let u = smoothstep(vec2(0.0), vec2(1.0), f);
    return mix(mix(a, b, u.x), mix(c, d, u.x), u.y);
}

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
    color = uneune(in.uv, globals.time);
    let color2 = block_noise(in.uv);
    //color = mix(color, color2, smoothstep(0.49, 0.51, (sin(globals.time * 2.0) + 1.0) / 2.));
    color = color;

    //let color = truche(in.uv);
    return vec4<f32>(color, 1.0);
}
