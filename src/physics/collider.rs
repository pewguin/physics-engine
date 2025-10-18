use std::ops::Mul;
use glam::{Mat3, Quat, Vec3};
use crate::rendering::transform::Transform;

const GEOMETRIC_EPSILON: f32 = 0.000001;
fn update_simplex(simplex: &mut Vec<Vec3>, dir: &mut Vec3) {
    match simplex.len() {
        2 => {
            let a = simplex[1];
            let b = simplex[0];
            *dir = if (b - a).dot(-a) > 0.0 {
                (b - a).cross((-a).cross(b - a))
            } else {
                -a
            };
        },
        3 => {
            let a = simplex[2];
            let b = simplex[1];
            let c = simplex[0];
            let norm = (b - a).cross(c - a);

            if norm.cross(c - a).dot(-a) > 0.0 { // CA or A region
                if (c - a).dot(-a) > 0.0 { // CA region
                    *simplex = vec![c, a];
                    *dir = (c - a).cross(-a).cross(c - a);
                } else { // A region
                    *simplex = vec![a];
                    *dir = -a;
                }
            } else { // BA, A, above, or below
                if (b - a).cross(norm).dot(-a) > 0.0 { // BA or A region
                    if (b - a).dot(-a) > 0.0 { // BA region
                        *simplex = vec![b, a];
                        *dir = (b - a).cross(-a).cross(b - a);
                    } else { // A region
                        *simplex = vec![a];
                        *dir = -a;
                    }
                } else { // Above or below
                    if norm.dot(-a) > 0.0 { // Above
                        *simplex = vec![a, b, c];
                        *dir = norm;
                    } else { // Below
                        *simplex = vec![a, b, c];
                        *dir = -norm;
                    }
                }
            }
        },
        4 => {
            let a = simplex[3];
            let b = simplex[2];
            let c = simplex[1];
            let d = simplex[0];

            let ac = c - a;
            let ab = b - a;
            let ad = d - a;
            let abc = ab.cross(ac).normalize();
            let abd = ab.cross(ad).normalize();
            let acd = ac.cross(ad).normalize();

            let dist_abc = (-abc).dot(a).abs();
            let dist_abd = (-abd).dot(a).abs();
            let dist_acd = (-acd).dot(a).abs();

            if dist_abc <= dist_abd && dist_abc <= dist_acd {
                *simplex = vec![c, b, a];
            } else if dist_abd <= dist_acd {
                *simplex = vec![d, b, a];
            } else {
                *simplex = vec![d, c, a];
            }
            update_simplex(simplex, dir);
        },
        len => panic!("Invalid simplex length of {}", len)
    }
}
fn simplex_contains_origin(simplex: &Vec<Vec3>) -> bool{
    match simplex.len() {
        1 => {
            let abs = simplex[0].abs();
            abs.x < GEOMETRIC_EPSILON &&
                abs.y < GEOMETRIC_EPSILON &&
                abs.z < GEOMETRIC_EPSILON
        },
        2 => {
            let a = simplex[1];
            let b = simplex[0];
            let abs = b.cross(a).abs();
            abs.x < GEOMETRIC_EPSILON &&
                abs.y < GEOMETRIC_EPSILON &&
                abs.z < GEOMETRIC_EPSILON
        },
        3 => {
            let a = simplex[2];
            let b = simplex[1];
            let c = simplex[0];

            let v0 = b - a;
            let v1 = c - a;
            let v2 = -a;

            let dot00 = v0.dot(v0);
            let dot01 = v0.dot(v1);
            let dot11 = v1.dot(v1);
            let dot02 = v0.dot(v2);
            let dot12 = v1.dot(v2);

            let inv_denom = 1.0 / (dot00 * dot11 - dot01 * dot01);
            let u = (dot11 * dot02 - dot01 * dot12) * inv_denom;
            let v = (dot00 * dot12 - dot01 * dot02) * inv_denom;
            
            u >= -GEOMETRIC_EPSILON && v >= -GEOMETRIC_EPSILON && (u + v) <= 1.0 + GEOMETRIC_EPSILON
        },
        4 => {
            let a = simplex[3];
            let b = simplex[2];
            let c = simplex[1];
            let d = simplex[0];

            let faces = [
                (a, b, c, d),
                (a, b, d, c),
                (a, c, d, b),
                (b, c, d, a),
            ];

            for (v0, v1, v2, opposite) in faces {
                let n = (v1 - v0).cross(v2 - v0);

                if n.length_squared() < GEOMETRIC_EPSILON * GEOMETRIC_EPSILON {
                    return false;
                }

                let sign_origin = n.dot(-v0);
                let sign_opposite = n.dot(opposite - v0);

                if sign_origin * sign_opposite < -GEOMETRIC_EPSILON {
                    return false;
                }
            }
            true
        },
        len => panic!("Invalid simplex length of {}", len)
    }
}
pub trait ColliderShape {
    // Function to return point on shape the furthest along a direction
    fn support(&self, dir: Vec3) -> Vec3;
    fn get_aabb(&self) -> AABB;
    fn transform(&self, transform: &Transform) -> Box<dyn ColliderShape>;
    fn collides_with(&self, other: Box<dyn ColliderShape>) -> Option<Vec<Vec3>> {
        let mut dir = Vec3::X;
        let mut simplex = Vec::with_capacity(4);
        simplex.push(self.support(dir) - other.support(-dir));
        dir = -simplex[0];
        loop {
            let new = self.support(dir) - other.support(-dir);

            if new.dot(dir) <= 0.0 {
                return None;
            }
            simplex.push(new);
            if simplex_contains_origin(&mut simplex) {
                return Some(simplex);
            }
            update_simplex(&mut simplex, &mut dir);
        }
    }
}
#[derive(Clone, Copy, Debug)]
pub struct Sphere {
    pub radius: f32,
    pub center: Vec3,
}
impl ColliderShape for Sphere {
    fn support(&self, dir: Vec3) -> Vec3 {
        self.center + dir.normalize() * self.radius
    }
    fn get_aabb(&self) -> AABB {
        AABB {
            min: Vec3::splat(-self.radius),
            max: Vec3::splat(self.radius),
        }
    }
    fn transform(&self, transform: &Transform) -> Box<dyn ColliderShape> {
        // Ellipses are not currently supported, so scale is interpreted as its max value
        Box::new(Self {
            radius: transform.scale.max_element(),
            center: self.center + transform.pos,
        })
    }
}
#[derive(Clone, Copy, Debug)]
pub struct AABB {
    pub min: Vec3,
    pub max: Vec3,
}
impl ColliderShape for AABB {
    fn support(&self, dir: Vec3) -> Vec3 {
        Vec3::new(
            if dir.x >= 0.0 { self.max.x } else { self.min.x },
            if dir.y >= 0.0 { self.max.y } else { self.min.y },
            if dir.z >= 0.0 { self.max.z } else { self.min.z },
        )
    }
    fn get_aabb(&self) -> AABB {
        *self
    }
    fn transform(&self, transform: &Transform) -> Box<dyn ColliderShape> {
        todo!()
    }
}
#[derive(Clone, Copy, Debug)]
pub struct Box3 {
    pub center: Vec3,
    pub rotation: Quat,
    pub half_extents: Vec3,
}
impl ColliderShape for Box3 {
    fn support(&self, dir: Vec3) -> Vec3 {
        let dir_local = self.rotation.conjugate() * dir;
        let extreme_local = Vec3::new(
            if dir_local.x >= 0.0 { self.half_extents.x } else { -self.half_extents.x },
            if dir_local.y >= 0.0 { self.half_extents.y } else { -self.half_extents.y },
            if dir_local.z >= 0.0 { self.half_extents.z } else { -self.half_extents.z },
        );
        self.rotation * extreme_local + self.center
    }
    fn get_aabb(&self) -> AABB {
        let rot = Mat3::from_quat(self.rotation);
        let abs_rot = Mat3::from_cols(
            rot.x_axis.abs(),
            rot.y_axis.abs(),
            rot.z_axis.abs(),
        );
        let extents = abs_rot * self.half_extents;
        AABB {
            min: self.center - extents,
            max: self.center + extents,
        }
    }
    fn transform(&self, transform: &Transform) -> Box<dyn ColliderShape> {
        Box::new(Self {
            center: self.center + transform.pos,
            rotation: transform.rot * self.rotation,
            half_extents: self.half_extents * transform.scale,
        })
    }
}