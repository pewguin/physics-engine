use std::collections::HashMap;
use glam::Vec3;
use crate::rendering::mesh::Mesh;
use crate::rendering::renderer;

pub struct World {
    positions: HashMap<u32, Vec3>,
    velocity: HashMap<u32, Vec3>,
    meshes: HashMap<u32, Mesh>,
}
impl World {
    pub fn new() -> Self {
        Self {
            positions: HashMap::new(),
            velocity: HashMap::new(),
            meshes: HashMap::new(),
        }
    }
}