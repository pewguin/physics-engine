use glam::{Mat4, Quat, Vec3};
use wgpu::Buffer;
use crate::rendering::renderer::Renderer;
use crate::rendering::transform::Transform;
use crate::rendering::vertex::Vertex;

const CUBE_VERTECIES: [Vertex; 24] = [
    Vertex { position: [-1.0, -1.0,  1.0], uv: [0.0, 0.0] },
    Vertex { position: [ 1.0, -1.0,  1.0], uv: [1.0, 0.0] },
    Vertex { position: [ 1.0,  1.0,  1.0], uv: [1.0, 1.0] },
    Vertex { position: [-1.0,  1.0,  1.0], uv: [0.0, 1.0] },

    Vertex { position: [ 1.0, -1.0, -1.0], uv: [0.0, 0.0] },
    Vertex { position: [-1.0, -1.0, -1.0], uv: [1.0, 0.0] },
    Vertex { position: [-1.0,  1.0, -1.0], uv: [1.0, 1.0] },
    Vertex { position: [ 1.0,  1.0, -1.0], uv: [0.0, 1.0] },

    Vertex { position: [-1.0, -1.0, -1.0], uv: [0.0, 0.0] },
    Vertex { position: [-1.0, -1.0,  1.0], uv: [1.0, 0.0] },
    Vertex { position: [-1.0,  1.0,  1.0], uv: [1.0, 1.0] },
    Vertex { position: [-1.0,  1.0, -1.0], uv: [0.0, 1.0] },

    Vertex { position: [ 1.0, -1.0,  1.0], uv: [0.0, 0.0] },
    Vertex { position: [ 1.0, -1.0, -1.0], uv: [1.0, 0.0] },
    Vertex { position: [ 1.0,  1.0, -1.0], uv: [1.0, 1.0] },
    Vertex { position: [ 1.0,  1.0,  1.0], uv: [0.0, 1.0] },

    Vertex { position: [-1.0,  1.0,  1.0], uv: [0.0, 0.0] },
    Vertex { position: [ 1.0,  1.0,  1.0], uv: [1.0, 0.0] },
    Vertex { position: [ 1.0,  1.0, -1.0], uv: [1.0, 1.0] },
    Vertex { position: [-1.0,  1.0, -1.0], uv: [0.0, 1.0] },

    Vertex { position: [-1.0, -1.0, -1.0], uv: [0.0, 0.0] },
    Vertex { position: [ 1.0, -1.0, -1.0], uv: [1.0, 0.0] },
    Vertex { position: [ 1.0, -1.0,  1.0], uv: [1.0, 1.0] },
    Vertex { position: [-1.0, -1.0,  1.0], uv: [0.0, 1.0] },
];
const CUBE_INDECIES: [u16; 36] = [
    0,  1,  2,  0,  2,  3,   // Front
    4,  5,  6,  4,  6,  7,   // Back
    8,  9, 10,  8, 10, 11,   // Left
    12, 13, 14, 12, 14, 15,   // Right
    16, 17, 18, 16, 18, 19,   // Top
    20, 21, 22, 20, 22, 23,   // Bottom
];
#[derive(Clone)]
pub struct Mesh {
    pub vertex_buffer: Buffer,
    pub vertex_count: u32,
    pub index_buffer: Buffer,
    pub index_count: u32,
    pub uniform_buffer: Buffer,
    pub bind_group: wgpu::BindGroup,
}

impl Mesh {
    pub fn create_cube(renderer: &mut Renderer) -> Self {
        renderer.create_mesh(CUBE_VERTECIES.to_vec(), CUBE_INDECIES.to_vec())
    }
    pub fn apply_transform(&mut self, transform: Transform) {

    }
}