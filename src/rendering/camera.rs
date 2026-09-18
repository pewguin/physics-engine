use glam::{Mat4, Quat, Vec3};

use crate::rendering::transform::Transform;

pub struct Camera {
    pub projection_matrix: Mat4,
    pub transform: Transform,
}

impl Camera {
    pub fn new() -> Self {
        let projection_matrix = Mat4::perspective_rh_gl(
            90.0_f32.to_radians(),
            1.0,
            0.0001,
            3000.0,
        );

        let transform = Transform {
            scale: Vec3::ONE,
            rot: Quat::IDENTITY,
            pos: Vec3::ZERO,
        };

        Self {
            projection_matrix,
            transform,
        }
    }

    pub fn update_proj_matrix(&mut self, width: f32, height: f32) {
        self.projection_matrix = Mat4::perspective_rh_gl(
            90.0_f32.to_radians(),
            width / height,
            0.0001,
            3000.0,
        );
    }
}
