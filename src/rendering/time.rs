use bytemuck::{Pod, Zeroable};
use wgpu::{vertex_attr_array, BufferAddress, VertexBufferLayout, VertexStepMode};

#[repr(C)]
#[derive(Pod, Zeroable, Debug, Clone, Copy)]
pub struct Time {
    pub total: f32,
    pub delta: f32,
}

impl Time {

}