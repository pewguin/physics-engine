use glam::{Mat4, Quat, Vec3};

#[derive(Clone)]
pub struct Transform {
    pub scale: Vec3,
    pub rot: Quat,
    pub pos: Vec3,
}

impl Transform {
    pub fn get_transform(&self) -> Mat4 {
        Mat4::from_scale_rotation_translation(self.scale, self.rot, self.pos)
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self { scale: Vec3::ONE, rot: Quat::IDENTITY, pos: Vec3::ZERO }
    }
}