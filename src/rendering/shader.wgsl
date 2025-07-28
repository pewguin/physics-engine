struct Uniform {
    model: mat4x4<f32>
}

struct Time {
    delta: f32,
    total: f32,
}

@group(0) @binding(0)
var<uniform> uniforms: Uniform;

@group(1) @binding(0)
var<uniform> time: Time;

struct VertexInput {
    @location(0) pos: vec3<f32>,
    @location(1) uv: vec2<f32>
}

struct VertexOutput {
    @builtin(position) clip_pos: vec4<f32>,
    @location(0) uv: vec2<f32>
}

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    let local = vec4<f32>(in.pos * (-cos(time.total * 3.0) + 1.0) / 2.0, 1.0);
    let clip = uniforms.model * local;

    var out: VertexOutput;
    out.clip_pos = clip;
    out.uv = in.uv;
    return out;
}

@fragment
fn fs_main(@location(0) uv: vec2<f32>) -> @location(0) vec4<f32> {
    let b = (-cos(time.total * 3.0) + 1.0) / 2.0;
    return vec4<f32>(uv, b, 1.0);
}