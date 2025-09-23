use std::ffi::c_void;
use std::ops;
use bytemuck::{Pod, Zeroable};
use wgpu::{vertex_attr_array, BufferAddress, VertexBufferLayout, VertexStepMode};

#[repr(C)]
#[derive(Pod, Zeroable, Debug, Clone, Copy)]
pub struct Vertex {
    pub position: [f32; 3],
    pub uv: [f32; 2],
}

impl Vertex {
    pub const LAYOUT: VertexBufferLayout<'_> = VertexBufferLayout {
        array_stride: size_of::<Self>() as BufferAddress,
        step_mode: VertexStepMode::Vertex,
        attributes: &vertex_attr_array![
            0 => Float32x3,
            1 => Float32x2,
        ],
    };
}

impl ops::Mul<f32> for Vertex {
    type Output = Vertex;
    fn mul(self, rhs: f32) -> Self::Output {
        Self {
            position: [
                self.position[0] * rhs,
                self.position[1] * rhs,
                self.position[2] * rhs,
            ],
            uv: self.uv
        }
    }
}