use std::fmt::{format, Display};
use glam::{Mat3, Quat, Vec3};
use crate::rendering::transform::Transform;

#[derive(Debug, Copy, Clone)]
pub enum ColliderShape {
    Sphere(Sphere),
    AABB(AABB),
    ColliderBox(ColliderBox),
    Capsule(Capsule),
}

#[derive(Debug, Copy, Clone)]
pub struct Sphere {
    pub center: Vec3,
    pub radius: f32,
}
impl Sphere {
    pub fn new(center: Vec3, radius: f32) -> Self {
        Self { center, radius }
    }
}
#[derive(Debug, Copy, Clone)]
pub struct AABB {
    pub min: Vec3,
    pub max: Vec3,
}
impl AABB {
    pub fn new(min: Vec3, max: Vec3) -> Self {
        Self { min, max }
    }
}
#[derive(Debug, Copy, Clone)]
pub struct ColliderBox {
    pub center: Vec3,
    pub size: Vec3,
    pub rot: Quat,
}
impl ColliderBox {
    pub fn new(center: Vec3, size: Vec3, rot: Quat) -> Self {
        Self { center, size, rot }
    }
}
#[derive(Debug, Copy, Clone)]
pub struct Capsule {
    pub center: Vec3,
    pub radius: f32,
    pub height: f32,
    pub rot: Quat,
}
impl Capsule {
    pub fn new(center: Vec3, radius: f32, height: f32, rot: Quat) -> Self {
        Self { center, radius, height, rot }
    }
}

impl ColliderShape {
    pub fn collides_with(&self, other: &ColliderShape) -> bool {
        match (self, other) {
            (ColliderShape::Sphere(a), ColliderShape::Sphere(b)) => Self::sphere_collision(a, b),
            (ColliderShape::AABB(a), ColliderShape::AABB(b)) => Self::aabb_collision(a, b),
            (ColliderShape::ColliderBox(a), ColliderShape::ColliderBox(b)) => Self::box_collision(a, b),
            (ColliderShape::Capsule(a), ColliderShape::Capsule(b)) => Self::capsule_collision(a, b),
            (ColliderShape::Sphere(a), ColliderShape::ColliderBox(b)) |
            (ColliderShape::ColliderBox(b), ColliderShape::Sphere(a)) => Self::sphere_box_collision(a, b),
            (ColliderShape::Sphere(a), ColliderShape::AABB(b)) |
            (ColliderShape::AABB(b), ColliderShape::Sphere(a)) => Self::sphere_aabb_collision(a, b),
            (ColliderShape::Sphere(a), ColliderShape::Capsule(b)) |
            (ColliderShape::Capsule(b), ColliderShape::Sphere(a)) => Self::sphere_capsule_collision(a, b),
            (ColliderShape::AABB(a), ColliderShape::ColliderBox(b)) |
            (ColliderShape::ColliderBox(b), ColliderShape::AABB(a)) => Self::aabb_box_collision(a, b),
            (ColliderShape::AABB(a), ColliderShape::Capsule(b)) |
            (ColliderShape::Capsule(b), ColliderShape::AABB(a)) => Self::aabb_capsule_collision(a, b),
            (ColliderShape::ColliderBox(a), ColliderShape::Capsule(b)) |
            (ColliderShape::Capsule(b), ColliderShape::ColliderBox(a)) => Self::box_capsule_collision(a, b),
        }
    }
    fn sphere_collision(a: &Sphere, b: &Sphere) -> bool {
        a.radius + b.radius >= a.center.distance(b.center)
    }
    fn aabb_collision(a: &AABB, b: &AABB) -> bool {
        (a.min.x <= b.max.x && a.max.x >= b.min.x) &&
            (a.min.y <= b.max.y && a.max.y >= b.min.y) &&
            (a.min.z <= b.max.z && a.max.z >= b.min.z)
    }
    fn box_collision(a: &ColliderBox, b: &ColliderBox) -> bool {
        let a_half = a.size * 0.5;
        let b_half = b.size * 0.5;

        let a_rot = Mat3::from_quat(a.rot);
        let b_rot = Mat3::from_quat(b.rot);

        let r = a_rot.transpose() * b_rot;

        let t_world = b.center - a.center;
        let t = a_rot.transpose() * t_world;

        let abs_r = Mat3::from_cols(
            r.x_axis.abs() + Vec3::splat(1e-6),
            r.y_axis.abs() + Vec3::splat(1e-6),
            r.z_axis.abs() + Vec3::splat(1e-6),
        );

        let r_at = |row: usize, col: usize| match col {
            0 => r.x_axis[row],
            1 => r.y_axis[row],
            _ => r.z_axis[row],
        };
        let abs_r_at = |row: usize, col: usize| match col {
            0 => abs_r.x_axis[row],
            1 => abs_r.y_axis[row],
            _ => abs_r.z_axis[row],
        };

        for i in 0..3 {
            let ra = a_half[i];
            let rb = b_half.x * abs_r_at(i, 0)
                + b_half.y * abs_r_at(i, 1)
                + b_half.z * abs_r_at(i, 2);
            if t[i].abs() > ra + rb {
                return false;
            }
        }

        for i in 0..3 {
            let ra = a_half.x * abs_r_at(0, i)
                + a_half.y * abs_r_at(1, i)
                + a_half.z * abs_r_at(2, i);
            let rb = b_half[i];
            let t_proj = t.x * r_at(0, i) + t.y * r_at(1, i) + t.z * r_at(2, i);
            if t_proj.abs() > ra + rb {
                return false;
            }
        }

        for i in 0..3 {
            for j in 0..3 {
                let t_proj = t[(i + 1) % 3] * r_at((i + 2) % 3, j)
                    - t[(i + 2) % 3] * r_at((i + 1) % 3, j);

                let ra = a_half[(i + 1) % 3] * abs_r_at((i + 2) % 3, j)
                    + a_half[(i + 2) % 3] * abs_r_at((i + 1) % 3, j);

                let rb = b_half[(j + 1) % 3] * abs_r_at(i, (j + 2) % 3)
                    + b_half[(j + 2) % 3] * abs_r_at(i, (j + 1) % 3);

                if t_proj.abs() > ra + rb {
                    return false;
                }
            }
        }

        true
    }


    fn capsule_collision(a: &Capsule, b: &Capsule) -> bool {
        todo!()
    }
    fn sphere_box_collision(a: &Sphere, b: &ColliderBox) -> bool {
        todo!()
    }

    fn sphere_aabb_collision(a: &Sphere, b: &AABB) -> bool {
        todo!()
    }

    fn sphere_capsule_collision(a: &Sphere, b: &Capsule) -> bool {
        todo!()
    }

    fn aabb_box_collision(a: &AABB, b: &ColliderBox) -> bool {
        todo!()
    }

    fn aabb_capsule_collision(a: &AABB, b: &Capsule) -> bool {
        todo!()
    }

    fn box_capsule_collision(a: &ColliderBox, b: &Capsule) -> bool {
        todo!()
    }

    pub fn calculate_aabb(&self) -> AABB {
        match self {
            ColliderShape::Sphere(s) => {
                let v1 = s.center + Vec3::splat(s.radius);
                let v2 = s.center - Vec3::splat(s.radius);
                AABB::new(v1, v2)
            }
            ColliderShape::AABB(aabb) => { aabb.clone() }
            ColliderShape::ColliderBox(b) => {
                let half = b.size * 0.5;
                let rot = Mat3::from_quat(b.rot);

                let abs_rot = Mat3::from_cols(
                    rot.x_axis.abs(),
                    rot.y_axis.abs(),
                    rot.z_axis.abs(),
                );

                let extents = abs_rot * half;

                AABB::new(b.center - extents, b.center + extents)
            }
            ColliderShape::Capsule(_) => {
                todo!();
            }
        }
    }
    pub fn transform_collider(shape: &ColliderShape, transform: &Transform) -> Option<ColliderShape> {
        match shape {
            ColliderShape::Sphere(s) => {
                let mut s = s.clone();
                s.center += transform.pos;
                s.radius *= transform.scale.x; // TODO: Support ellipses
                return Some(ColliderShape::Sphere(s));
            }
            ColliderShape::AABB(aabb) => {
                // let mut aabb = aabb.clone();
                // aabb.min += transform.pos;
                // aabb.max += transform.pos;
                // aabb.min
                // return Some(ColliderShape::AABB(aabb));
                todo!()
            }
            ColliderShape::ColliderBox(b) => {
                let mut b = b.clone();
                b.center += transform.pos;
                b.rot *= transform.rot;
                b.size.x *= transform.scale.x;
                b.size.y *= transform.scale.y;
                b.size.z *= transform.scale.z;

                return Some(ColliderShape::ColliderBox(b));
            }
            ColliderShape::Capsule(c) => {
                todo!()
            }
        }
    }
}