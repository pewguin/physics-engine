use wgpu::Buffer;

pub struct BufferedMesh {
    pub vertex_buffer: Buffer,
    pub index_buffer: Buffer,
    pub transform_buffer: Buffer,
    pub bind_group: wgpu::BindGroup,
}