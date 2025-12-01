use glam::Vec3;
use crate::rendering::vertex::Vertex;

const CUBE_VERTEXES: [Vertex; 24] = [
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
const CUBE_INDICES: [u16; 36] = [
    0,  1,  2,  0,  2,  3,   // Front
    4,  5,  6,  4,  6,  7,   // Back
    8,  9, 10,  8, 10, 11,   // Left
    12, 13, 14, 12, 14, 15,   // Right
    16, 17, 18, 16, 18, 19,   // Top
    20, 21, 22, 20, 22, 23,   // Bottom
];
#[derive(Clone)]
pub struct Mesh {
    pub vertexes: Vec<Vertex>,
    pub indices: Vec<u16>,
}

impl Mesh {
    pub fn cube() -> Self {
        Self {
            vertexes: CUBE_VERTEXES.to_vec().iter().map(|v| *v * 0.5).collect(),
            indices: CUBE_INDICES.to_vec(),
        }
    }
}