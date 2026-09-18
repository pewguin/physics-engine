struct Matrix {
    model: mat4x4<f32>
}

// G0: Frame Bindings
//   B0: View matrix
//   B1: Projection matrix
//   B2: Viewport
// G1: Mesh Bindings
//   B0: Model matrix
//   B1: Positions array
// G2: Material Bindings (NOT BOUND)
//   B0: Texture view
//   B1: Sampler

@group(0) @binding(0) var<uniform> view_matrix: Matrix;
@group(0) @binding(1) var<uniform> projection_matrix: Matrix;
@group(0) @binding(2) var<uniform> viewport: vec2<f32>;

@group(1) @binding(0) var<uniform> model_matrix: Matrix;
@group(1) @binding(1) var<storage, read> positions: array<f32>;

struct VertexOutput {
    @builtin(position) clip_pos: vec4<f32>,
    @location(0) @interpolate(flat) v0: vec2<f32>,
    @location(1) @interpolate(flat) v1: vec2<f32>,
    @location(2) @interpolate(flat) v2: vec2<f32>,
}

fn to_screen(clip: vec4<f32>) -> vec2<f32> {
    let perspective = clip.xy / clip.w;
    return (perspective * vec2<f32>(0.5, -0.5) + vec2<f32>(0.5)) * viewport;
}

fn get_pos(vert_idx: u32) -> vec3<f32> {
    let base = vert_idx * 3u;
    return vec3<f32>(positions[base], positions[base + 1u], positions[base + 2u]);
}

@vertex
fn vs_main(@builtin(vertex_index) idx: u32) -> VertexOutput {
    let tri = idx / 3u;
    let base = tri * 3u;
    let model_projected = projection_matrix.model * view_matrix.model * model_matrix.model;

    let clip_0 = model_projected * vec4<f32>(get_pos(base), 1.0);
    let clip_1 = model_projected * vec4<f32>(get_pos(base + 1u), 1.0);
    let clip_2 = model_projected * vec4<f32>(get_pos(base + 2u), 1.0);
    var clip_pos = array<vec4<f32>, 3>(clip_0, clip_1, clip_2);

    var out: VertexOutput;
    out.clip_pos = clip_pos[idx % 3u];
    out.v0 = to_screen(clip_0);
    out.v1 = to_screen(clip_1);
    out.v2 = to_screen(clip_2);
    return out;
}

fn edge_dist(p: vec2<f32>, a: vec2<f32>, b: vec2<f32>) -> f32 {
    let ab = b - a;
    let ap = p - a;
    return abs(ab.x * ap.y - ab.y * ap.x) / length(ab);
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let p = in.clip_pos.xy;
    let dist = min(edge_dist(p, in.v1, in.v2),
               min(edge_dist(p, in.v2, in.v0),
                   edge_dist(p, in.v0, in.v1)));

    let half_width = 0.75;
    let alpha = 1.0 - smoothstep(half_width, half_width + 1.0, dist);

    if (alpha <= 0.001) {
        discard;
    }
    return vec4<f32>(0.0, 1.0, 0.0, alpha);
}
