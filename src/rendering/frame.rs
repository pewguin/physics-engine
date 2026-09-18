use wgpu::{CommandEncoder, SurfaceTexture, TextureView};

pub struct Frame {
    pub surface_texture: SurfaceTexture,
    pub view: TextureView,
    pub encoder: CommandEncoder,
}
