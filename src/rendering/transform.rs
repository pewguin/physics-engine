use glam::{Mat4, Quat, Vec3};

#[derive(Clone)]
pub struct Transform {
    pub scale: Vec3,
    pub rot: Quat,
    pub pos: Vec3,
}

impl Transform {
    pub fn as_matrix(&self) -> Mat4 {
        Mat4::from_scale_rotation_translation(self.scale, self.rot, self.pos)
    }

    pub fn with_pos(self, pos: Vec3) -> Transform {
        Self {
            scale: self.scale,
            rot: self.rot,
            pos,
        }
    }

    pub fn with_rot(self, rot: Quat) -> Transform {
        Self {
            scale: self.scale,
            rot,
            pos: self.pos,
        }
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self { scale: Vec3::ONE, rot: Quat::IDENTITY, pos: Vec3::ZERO }
    }
}
