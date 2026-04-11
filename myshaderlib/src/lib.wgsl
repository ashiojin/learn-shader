#define_import_path mylib

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
