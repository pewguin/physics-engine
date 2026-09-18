use wgpu::{BindGroup, Buffer};

pub struct BufferedWireframeMesh {
    pub transform_buffer: Buffer,
    pub positions_buffer: Buffer,
    pub position_count: u32,

    pub mesh_bind_group: BindGroup,
}
