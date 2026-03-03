use std::rc::Rc;

use glam::Vec3;
use wgpu::Color;

use crate::{physics::mesh::Mesh, rendering::renderer::Renderer};

pub enum DebugCommand {
    Pause,
    RenderWireframe(Mesh, Color),
    Log(String),
}

pub struct Debugger {
    pub cmds: Vec<DebugCommand>,
}

impl Debugger {
    pub fn new() -> Self {
        Self {
            cmds: Vec::new(),
        }
    }

    pub fn pause(&mut self) {
        self.cmds.push(DebugCommand::Pause);
    }

    pub fn draw_wireframe(&mut self, mesh: Mesh, color: Color) {
        self.cmds.push(DebugCommand::RenderWireframe(mesh, color));
    }

    pub fn log(&mut self, msg: String) {
        self.cmds.push(DebugCommand::Log(msg));
    }
}
