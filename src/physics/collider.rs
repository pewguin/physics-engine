use std::collections::HashSet;
use std::hash::Hash;
use std::mem::swap;
use std::ops::Mul;
use glam::{Mat3, Quat, Vec3};
use crate::rendering::transform::Transform;

const GEOMETRIC_EPSILON: f32 = 0.000001;
fn support(a: &dyn ColliderShape, b: &dyn ColliderShape, dir: Vec3) -> Vec3 {
    a.support(dir) - b.support(-dir)
}
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
fn expand_simplex(mut simplex: Vec<Vec3>, b1: &dyn ColliderShape, b2: &dyn ColliderShape) -> Vec<Vec3> {
    match simplex.len() {
        3 => {
            let a = simplex[0];
            let b = simplex[1];
            let c = simplex[2];

            let mut normal = (b - c).cross(c - a).normalize();

            if normal.dot(a) > 0.0 {
                simplex.swap(1, 2);
                normal = -normal;
            }
            let d = support(b1, b2, normal);
            simplex.push(d);
            simplex
        }
        4 => {
            simplex
        }
        _ => panic!("expected simplex of size 3 or 4")
    }
}
fn distance_to_origin(a: Vec3, b: Vec3, c: Vec3) -> f32 {
    let mut norm = (b - a).cross(c - a).normalize();
    if norm.dot(a) < 0.0 {
        norm = -norm;
    }
    norm.dot(a)
}
struct Polytope {
    vertexes: Vec<Vec3>,
    indexes: Vec<usize>,
}
impl Polytope {
    pub fn from_simplex(simplex: Vec<Vec3>, b1: &dyn ColliderShape, b2: &dyn ColliderShape) -> Self {
        Self {
            vertexes: expand_simplex(simplex, b1, b2),
            indexes: vec![
                0, 1, 2,
                0, 3, 1,
                1, 3, 2,
                2, 3, 0,
            ],
        }
    }
    fn closest_face_to_origin(&self) -> (Vec3, Vec3, Vec3) {
        let mut closest_face: Option<(Vec3, Vec3, Vec3)> = None;
        let mut closest_distance = f32::MAX;
        for face in self.indexes.chunks(3) {
            let a = self.vertexes[face[0]];
            let b = self.vertexes[face[1]];
            let c = self.vertexes[face[2]];

            let dist = distance_to_origin(a, b, c);

            if dist < closest_distance {
                closest_distance = dist;
                closest_face = Some((a, b, c))
            }
        }
        closest_face.unwrap()
    }
    pub fn project_origin_to_closest_face(&self) -> Vec3 {
        let (a, b, c) = self.closest_face_to_origin();
        let norm = (b - a).cross(c - a).normalize();
        let dist = distance_to_origin(a, b, c);
        norm * (-dist)
    }
    pub fn add_vertex(&mut self, vert: Vec3) {
        let mut horizon_edges: HashSet<(usize, usize)> = HashSet::new();
        let mut faces_to_remove = Vec::new();

        // find visible faces
        for (i, face) in self.indexes.chunks(3).enumerate() {
            let a = self.vertexes[face[0]];
            let b = self.vertexes[face[1]];
            let c = self.vertexes[face[2]];

            let norm = (b - a).cross(c - a).normalize();
            if (vert - a).dot(norm) > 0.0 {
                faces_to_remove.push(i);
            }
        }

        // process edges
        for &face_idx in faces_to_remove.iter().rev() {
            let tri = face_idx * 3;
            let a = self.indexes[tri];
            let b = self.indexes[tri + 1];
            let c = self.indexes[tri + 2];

            let face_edges = [
                (a, b),
                (b, c),
                (c, a),
            ];

            for edge in face_edges {
                let rev = (edge.1, edge.0);

                if horizon_edges.contains(&rev) {
                    horizon_edges.remove(&rev);
                } else {
                    horizon_edges.insert(rev);
                }
            }

            self.indexes.drain(tri..tri+3);
        }

        let v_index = self.vertexes.len();
        self.vertexes.push(vert);

        for edge in horizon_edges {
            self.add_face(edge.0, edge.1, v_index);
        }
    }
    pub fn add_face(&mut self, a_i: usize, b_i: usize, c_i: usize) {
        let a = self.vertexes[a_i];
        let b = self.vertexes[b_i];
        let c = self.vertexes[c_i];

        let mut n = (b - a).cross(c - a).normalize();

        if n.dot(a) < 0.0 {
            self.indexes.extend_from_slice(&[a_i, c_i, b_i]);
        } else {
            self.indexes.extend_from_slice(&[a_i, b_i, c_i]);
        }
    }
}
fn project(a: Vec3, b: Vec3) -> Vec3 {
    todo!("proj fn")
}
// Uses expanding polytope algorithm
fn find_mvt(a: Box<dyn ColliderShape>, b: Box<dyn ColliderShape>, mut polytope: Polytope) -> Vec3 {
    loop {
        let proj = polytope.project_origin_to_closest_face();
        println!("proj: {:?}", proj);
        let support = support(a.as_ref(), b.as_ref(), proj);
        let abs = (project(support, proj) - proj).abs();
        if abs.x < GEOMETRIC_EPSILON &&
            abs.y < GEOMETRIC_EPSILON &&
            abs.z < GEOMETRIC_EPSILON {
            return proj;
        } else {
            polytope.add_vertex(proj);
        }
    }
}
pub trait ColliderShape {
    // Function to return point on shape the furthest along a direction
    fn support(&self, dir: Vec3) -> Vec3;
    fn get_aabb(&self) -> AABB;
    fn transform(&self, transform: &Transform) -> Box<dyn ColliderShape>;
}
// Uses GJK algorithm
pub fn collides_with(a: Box<dyn ColliderShape>, b: Box<dyn ColliderShape>) -> Option<Vec3> {
    let mut dir = Vec3::X;
    let mut simplex = Vec::with_capacity(4);
    simplex.push(support(a.as_ref(), b.as_ref(), dir));
    dir = -simplex[0];
    loop {
        let new = support(a.as_ref(), b.as_ref(), dir);

        if new.dot(dir) <= 0.0 {
            return None;
        }
        simplex.push(new);
        if simplex_contains_origin(&mut simplex) {
            let polytope = Polytope::from_simplex(simplex, a.as_ref(), b.as_ref());
            // return Some(find_mvt(a, b, polytope));
            return Some(Vec3::ZERO);
        }
        update_simplex(&mut simplex, &mut dir);
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
            min: Vec3::splat(-self.radius) + self.center,
            max: Vec3::splat(self.radius) + self.center,
        }
    }
    fn transform(&self, transform: &Transform) -> Box<dyn ColliderShape> {
        // Ellipses are not currently supported, so scale is interpreted as its max value
        Box::new(Self {
            radius: self.radius * transform.scale.max_element(),
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
        Box::new(Self {
            min: self.min + transform.pos,
            max: self.max + transform.pos,
        })
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
