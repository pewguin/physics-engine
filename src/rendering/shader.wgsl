struct Matrix {
    model: mat4x4<f32>
}

// G0: Texture Bindings
//   B0: Obama texture
//   B1: Obama view
// G1: Frame Bindings
//   B0: View matrix
//   B1: Projection matrix
// G2: Mesh Bindings
//   B0: Model matrix

@group(0) @binding(0)
var obama: texture_2d<f32>;
@group(0) @binding(1)
var obama_sampler: sampler;

@group(1) @binding(0)
var<uniform> view_matrix: Matrix;
@group(1) @binding(1)
var<uniform> projection_matrix: Matrix;

@group(2) @binding(0)
var<uniform> model_matrix: Matrix;

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
    let local = vec4<f32>(in.pos, 1.0);
    let clip = projection_matrix.model * view_matrix.model * model_matrix.model * local;

    var out: VertexOutput;
    out.clip_pos = clip;
    out.uv = in.uv;
    return out;
}

@fragment
fn fs_main(@location(0) uv: vec2<f32>) -> @location(0) vec4<f32> {
    let uv_flipped = vec2<f32>(uv.x, 1 - uv.y);
    return textureSample(obama, obama_sampler, uv_flipped);
}
