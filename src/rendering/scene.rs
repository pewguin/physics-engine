use std::collections::HashMap;

use wgpu::RenderPass;

use crate::{physics::world::World, rendering::{buffered_mesh::BufferedMesh, renderer::Renderer, wireframe_mesh::WireframeMesh}};

pub struct Scene {
    meshes: HashMap<u32, BufferedMesh>,
    wireframe_meshes: HashMap<u32, WireframeMesh>,
}

impl Scene {
    pub fn new() -> Self {
        Self {
            meshes: HashMap::new(),
            wireframe_meshes: HashMap::new(),
        }
    }

    pub fn register_mesh(&mut self, id: u32, mesh: BufferedMesh) {
        self.meshes.insert(id, mesh);
    }

    pub fn register_wireframe(&mut self, id: u32, mesh: WireframeMesh) {
        self.wireframe_meshes.insert(id, mesh);
    }

    pub fn render(&self, world: &World, renderer: &Renderer, render_pass: &mut RenderPass) {
        for (id, mesh) in &self.meshes {
            renderer.update_mesh(mesh, world.transforms.get(id).expect("IDs did not line up"));
            renderer.draw_mesh(render_pass, mesh);
        }
    }
}

