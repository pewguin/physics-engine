use wgpu::{BindGroup, Sampler, Texture};

pub struct Material {
    pub texture: Texture,
    pub sampler: Sampler,
    pub bind_group: BindGroup,
}

