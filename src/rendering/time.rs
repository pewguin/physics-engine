use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Pod, Zeroable, Debug, Clone, Copy)]
pub struct Time {
    pub total: f32,
    pub delta: f32,
}

impl Time {

}