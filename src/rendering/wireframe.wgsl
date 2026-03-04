struct Matrix {
    model: mat4x4<f32>
}

// G0: Frame Bindings
//   B0: View matrix
//   B1: Projection matrix
// G1: Mesh Bindings
//   B0: Model Matrix
// G2: 

@group(0) @binding(0)
var<uniform> view_matrix: Matrix;
@group(0) @binding(1)
var<uniform> projection_matrix: Matrix;

@group(1) @binding(0)
var<uniform> model_matrix: Matrix;

struct VertexInput {
    @location(0) pos: vec3<f32>,
    @location(1) bary: vec3<f32>
}

struct VertexOutput {
    @builtin(position) clip_pos: vec4<f32>,
    @location(0) @interpolate(linear) bary: vec3<f32>
}

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    let local = vec4<f32>(in.pos, 1.0);
    let clip = projection_matrix.model * view_matrix.model * model_matrix.model * local;

    var out: VertexOutput;
    out.clip_pos = clip;
    out.bary = in.bary;
    return out;
}

@fragment
fn fs_main(@location(0) @interpolate(linear) bary: vec3<f32>) -> @location(0) vec4<f32> {
    let d = fwidth(bary);
    let a3 = smoothstep(vec3<f32>(0.0), d * 1.5, bary);
    let edge = min(a3.x, min(a3.y, a3.z));
    let dist = 1.0 - edge;

    if (edge > 0.12) {
        discard;
    }

    return vec4<f32>(0.0, 1.0, 0.0, 1.0);
}
