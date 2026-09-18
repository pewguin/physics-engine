use std::rc::Rc;

use wgpu::Buffer;

use crate::rendering::{material::Material, transform::Transform};

pub struct BufferedMesh {
    pub vertex_buffer: Buffer,
    pub index_buffer: Buffer,
    pub transform_buffer: Buffer,

    pub index_count: u32,

    pub material: Rc<Material>,
    pub mesh_bind_group: wgpu::BindGroup,
}
