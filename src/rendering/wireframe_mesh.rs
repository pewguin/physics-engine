use wgpu::{BindGroup, Buffer};
use crate::{physics::mesh::Mesh, rendering::{buffered_mesh::BufferedMesh, vertex::Vertex, wireframe_vertex::WireframeVertex}};

pub struct WireframeMesh {
    pub vertexes: Vec<WireframeVertex>,
    pub vertex_buffer: Buffer,
    pub transform_buffer: Buffer,
    pub bind_group: BindGroup,
}

