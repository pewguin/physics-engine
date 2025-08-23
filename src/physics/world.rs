use std::collections::HashMap;
use glam::Vec3;
use crate::rendering::mesh::Mesh;
use crate::rendering::renderer;
use crate::rendering::transform::Transform;

pub struct World {
    pub transforms: HashMap<u32, Transform>,
    pub velocities: HashMap<u32, Vec3>,
    pub meshes: HashMap<u32, Mesh>,
}
impl World {
    pub fn new() -> Self {
        Self {
            transforms: HashMap::new(),
            velocities: HashMap::new(),
            meshes: HashMap::new(),
        }
    }
    pub fn add_mesh(&mut self, id: u32, position: Transform, velocity: Vec3, mesh: Mesh) {
        self.transforms.insert(id, position);
        self.velocities.insert(id, velocity);
        self.meshes.insert(id, mesh);
    }
}