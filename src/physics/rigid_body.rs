use glam::Vec3;
use crate::rendering::transform::Transform;

const GRAVITY: Vec3 = Vec3::new(0.0, -9.80665, 0.0);
pub enum CollisionType<'a> {
    Static,
    OtherRigidBody(&'a RigidBody),
}
pub struct RigidBody {
    pub velocity: Vec3,
    pub mass: f32,
    pub center_of_mass: Vec3,
    pub do_gravity: bool,
    pub restitution: f32,
}

impl RigidBody {
    pub fn new(center_of_mass: Vec3, do_gravity: bool, restitution: f32) -> RigidBody {
        Self {
            velocity: Vec3::ZERO,
            mass: 0.0,
            center_of_mass,
            do_gravity,
            restitution,
        }
    }
    pub fn step(&mut self, body_transform: &mut Transform, delta_time: f32) {
        let half_d_v = GRAVITY * delta_time * 0.5;
        if self.do_gravity {
            self.velocity += half_d_v;
        }
        body_transform.pos += self.velocity * delta_time;
        if self.do_gravity {
            self.velocity += half_d_v;
        }
    }
    pub fn collided(&mut self, other: &CollisionType) {
        self.velocity = -self.velocity * self.restitution;
    }
}
