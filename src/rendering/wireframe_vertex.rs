use bytemuck::{Pod, Zeroable};
use wgpu::{BufferAddress, VertexBufferLayout, VertexStepMode, vertex_attr_array};

#[repr(C)]
#[derive(Pod, Zeroable, Debug, Clone, Copy)]
pub struct WireframeVertex {
    pub position: [f32; 3],
}

impl WireframeVertex {
    pub const LAYOUT: VertexBufferLayout<'_> = VertexBufferLayout {
        array_stride: size_of::<Self>() as BufferAddress,
        step_mode: VertexStepMode::Vertex,
        attributes: &vertex_attr_array![
            0 => Float32x3,
        ],
    };
}
