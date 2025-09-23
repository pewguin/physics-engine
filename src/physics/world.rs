use crate::physics::mesh::Mesh;
use crate::rendering::transform::Transform;
use glam::Vec3;
use std::collections::HashMap;
use crate::physics::collider::ColliderShape;
use crate::physics::rigid_body::RigidBody;

pub struct World {
    pub transforms: HashMap<u32, Transform>,
    pub colliders: HashMap<u32, ColliderShape>,
    pub rigid_bodies: HashMap<u32, RigidBody>,
    pub meshes: HashMap<u32, Mesh>,
}
impl World {
    pub fn new() -> Self {
        Self {
            transforms: HashMap::new(),
            rigid_bodies: HashMap::new(),
            meshes: HashMap::new(),
            colliders: HashMap::new(),
        }
    }
    pub fn add_mesh(&mut self, id: u32, collider_shape: ColliderShape, position: Transform, mesh: Mesh, rigid_body: RigidBody) {
        self.transforms.insert(id, position);
        self.colliders.insert(id, collider_shape);
        self.rigid_bodies.insert(id, rigid_body);
        self.meshes.insert(id, mesh);
    }
    pub fn get_transformed_collider(&self, id: u32) -> Option<ColliderShape> {
        if let Some(col) = self.colliders.get(&id) {
            if let Some(trans) = self.transforms.get(&id) {
                return ColliderShape::transform_collider(col, trans)
            }
        }
        None
    }
    pub fn do_physics_step(&mut self, delta_time: f32) {
        for (id, rb) in self.rigid_bodies.iter_mut() {
            rb.step(self.transforms.get_mut(id).unwrap(), delta_time);
        }
    }
}