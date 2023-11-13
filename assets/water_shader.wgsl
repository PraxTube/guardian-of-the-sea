#import bevy_pbr::{mesh_view_bindings::globals, forward_io::VertexOutput}

const OCTAVE = 6;

struct Params {
    scale: f32,
    height: f32,
    tide: f32,
    foam_thickness: f32,
    time_scale: f32,
    water_depth: f32,
};

struct Color {
    w1: vec4<f32>,
    w2: vec4<f32>,
    foam: vec4<f32>,
};

@group(1) @binding(0) var<uniform> params: Params;
@group(1) @binding(1) var<uniform> colors: Color;

fn rand(uv: vec2<f32>) -> f32 {
    return (fract(sin(dot(uv.xy, vec2(23.53, 44.0))) * 42350.45));
}

fn noise(uv: vec2<f32>) -> f32 {
    let i: vec2<f32> = floor(uv);
    let j: vec2<f32> = fract(uv);

    let a: f32 = rand(i);
    let b: f32 = rand(i + vec2<f32>(1.0, 0.0));
    let c: f32 = rand(i + vec2<f32>(0.0, 1.0));
    let d: f32 = rand(i + vec2<f32>(1.0, 1.0));

    let blur: vec2<f32> = smoothstep(vec2<f32>(0.0, 0.0), vec2<f32>(1.0, 1.0), j);
    return mix(mix(a, b, blur.x), mix(c, d, blur.x), blur.y);
}

fn fbm(uv: vec2<f32>) -> f32 {
    var frequency: f32 = 3.0;
    var amplitude: f32 = 0.5;
    var value: f32 = 0.0;

    for (var i: i32 = 0; i < OCTAVE; i = i + 1) {
        value += noise(uv * frequency) * amplitude;
        frequency *= 2.0;
        amplitude *= 0.5;
    }
    return value;
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let pixelation: f32 = 25000.0;
    let UV = floor(in.uv * pixelation) / pixelation;
    var time: f32 = params.time_scale;

    let fbm_val_in_x: f32 = UV.x * params.scale + 0.2 * sin(0.3 * time) + 0.15 * time;
    let fbm_val_in_y: f32 = -0.05 * time + UV.y * params.scale + 0.1 * cos(0.68 * time);
    var fbm_val: f32 = fbm(vec2<f32>(fbm_val_in_x, fbm_val_in_y));

    var fbm_val_shadow: f32 = fbm(vec2<f32>(UV.x * params.scale + 0.2 * sin(-0.6 * time + 25.0 * UV.y) + 0.15 * time + 3.0, -0.05 * time + UV.y * params.scale + 0.13 * cos(-0.68 * time)) - 7.0 + 0.1 * sin(0.43 * time));
    var height: f32 = params.height + params.tide * sin(time + 5.0 * UV.x - 8.0 * UV.y);
    var shadow_height: f32 = params.height + params.tide * 1.3 * cos(time + 2.0 * UV.x - 2.0 * UV.y);

    var within_foam: f32 = step(height, fbm_val) * step(fbm_val, height + params.foam_thickness);
    var shadow: f32 = (1.0 - within_foam) * step(shadow_height, fbm_val_shadow) * step(fbm_val_shadow, shadow_height + params.foam_thickness * 0.7);

    var COLOR: vec4<f32> = within_foam * colors.foam + shadow * colors.w2 + ((1.0 - within_foam) * (1.0 - shadow)) * colors.w1;

    return COLOR;
}
