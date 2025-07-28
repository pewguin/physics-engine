use glam::{Mat4, Quat, Vec3};
use wgpu::Buffer;

pub struct Mesh {
    pub vertex_buffer: Buffer,
    pub vertex_count: u32,
    pub index_buffer: Buffer,
    pub index_count: u32,
    pub uniform_buffer: Buffer,
    pub bind_group: wgpu::BindGroup,
    
    pub pos: Vec3,
    pub rot: Quat,
    pub scale: Vec3,
}

impl Mesh {
    pub fn get_transform(&self) -> Mat4 {
         Mat4::from_scale_rotation_translation(self.scale, self.rot, self.pos)
    }
}