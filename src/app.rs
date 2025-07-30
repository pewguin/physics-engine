use crate::rendering::renderer::Renderer;
use anyhow::Result;
use futures::executor::block_on;
use std::sync::{Arc, RwLock};
use std::time::Instant;
use glam::{vec3, EulerRot, Quat, Vec3};
use wgpu::BufferUsages;
use wgpu::hal::DynCommandEncoder;
use wgpu::util::BufferInitDescriptor;
use winit::application::ApplicationHandler;
use winit::dpi::{LogicalPosition, PhysicalSize, Position, Size};
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::monitor::VideoModeHandle;
use winit::platform::x11::WindowAttributesExtX11;
use winit::window::{Fullscreen, Window, WindowAttributes, WindowId};
use crate::rendering::mesh::Mesh;
use crate::rendering::time::Time;
use crate::rendering::vertex::Vertex;

pub struct AppState {}
pub struct App<'a> {
    pub state: Arc<RwLock<AppState>>,
    pub renderer: Option<Renderer<'a>>,
    pub meshes: Vec<Mesh>,
    pub window: Option<Arc<Window>>,
    last_time: Instant,
    start_time: Instant,
}
impl App<'_> {
    pub fn new() -> Self {
        Self {
            state: Arc::new(RwLock::new(AppState {})),
            renderer: None,
            meshes: Vec::new(),
            window: None,
            last_time: Instant::now(),
            start_time: Instant::now(),
        }
    }
}
impl ApplicationHandler for App<'_> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window_attributes = WindowAttributes::default().with_name("renderer", "of things");
        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());
        self.window = Some(window.clone());
        let mut renderer = Renderer::new(
            window
        );

        // let vertices = vec![
        //     Vertex { position: [-1.0, -1.0,  1.0], uv: [1.0, 0.0, 1.0] },
        //     Vertex { position: [ 1.0, -1.0,  1.0], uv: [0.0, 1.0, 0.0] },
        //     Vertex { position: [ 1.0,  1.0,  1.0], uv: [1.0, 0.0, 0.0] },
        //     Vertex { position: [-1.0,  1.0,  1.0], uv: [1.0, 1.0, 0.0] },
        //     Vertex { position: [-1.0, -1.0, -1.0], uv: [1.0, 0.0, 0.0] },
        //     Vertex { position: [ 1.0, -1.0, -1.0], uv: [0.0, 1.0, 0.0] },
        //     Vertex { position: [ 1.0,  1.0, -1.0], uv: [0.0, 0.0, 1.0] },
        //     Vertex { position: [-1.0,  1.0, -1.0], uv: [1.0, 1.0, 0.0] },
        // ];
        // 
        // let indices = vec![
        //     // Front
        //     0, 1, 2,
        //     2, 3, 0,
        //     // Right
        //     1, 5, 6,
        //     6, 2, 1,
        //     // Back
        //     5, 4, 7,
        //     7, 6, 5,
        //     // Left
        //     4, 0, 3,
        //     3, 7, 4,
        //     // Top
        //     3, 2, 6,
        //     6, 7, 3,
        //     // Bottom
        //     4, 5, 1,
        //     1, 0, 4,
        // ];

        let vertices = vec![
            // Front face (z = 1.0)
            Vertex { position: [-1.0, -1.0,  1.0], uv: [0.0, 0.0] },
            Vertex { position: [ 1.0, -1.0,  1.0], uv: [1.0, 0.0] },
            Vertex { position: [ 1.0,  1.0,  1.0], uv: [1.0, 1.0] },
            Vertex { position: [-1.0,  1.0,  1.0], uv: [0.0, 1.0] },

            // Back face (z = -1.0)
            Vertex { position: [ 1.0, -1.0, -1.0], uv: [0.0, 0.0] },
            Vertex { position: [-1.0, -1.0, -1.0], uv: [1.0, 0.0] },
            Vertex { position: [-1.0,  1.0, -1.0], uv: [1.0, 1.0] },
            Vertex { position: [ 1.0,  1.0, -1.0], uv: [0.0, 1.0] },

            // Left face (x = -1.0)
            Vertex { position: [-1.0, -1.0, -1.0], uv: [0.0, 0.0] },
            Vertex { position: [-1.0, -1.0,  1.0], uv: [1.0, 0.0] },
            Vertex { position: [-1.0,  1.0,  1.0], uv: [1.0, 1.0] },
            Vertex { position: [-1.0,  1.0, -1.0], uv: [0.0, 1.0] },

            // Right face (x = 1.0)
            Vertex { position: [ 1.0, -1.0,  1.0], uv: [0.0, 0.0] },
            Vertex { position: [ 1.0, -1.0, -1.0], uv: [1.0, 0.0] },
            Vertex { position: [ 1.0,  1.0, -1.0], uv: [1.0, 1.0] },
            Vertex { position: [ 1.0,  1.0,  1.0], uv: [0.0, 1.0] },

            // Top face (y = 1.0)
            Vertex { position: [-1.0,  1.0,  1.0], uv: [0.0, 0.0] },
            Vertex { position: [ 1.0,  1.0,  1.0], uv: [1.0, 0.0] },
            Vertex { position: [ 1.0,  1.0, -1.0], uv: [1.0, 1.0] },
            Vertex { position: [-1.0,  1.0, -1.0], uv: [0.0, 1.0] },

            // Bottom face (y = -1.0)
            Vertex { position: [-1.0, -1.0, -1.0], uv: [0.0, 0.0] },
            Vertex { position: [ 1.0, -1.0, -1.0], uv: [1.0, 0.0] },
            Vertex { position: [ 1.0, -1.0,  1.0], uv: [1.0, 1.0] },
            Vertex { position: [-1.0, -1.0,  1.0], uv: [0.0, 1.0] },
        ];
        let indices = vec![
            0,  1,  2,  0,  2,  3,   // Front
            4,  5,  6,  4,  6,  7,   // Back
            8,  9, 10,  8, 10, 11,   // Left
            12, 13, 14, 12, 14, 15,   // Right
            16, 17, 18, 16, 18, 19,   // Top
            20, 21, 22, 20, 22, 23,   // Bottom
        ];

        let mut mesh = renderer.create_mesh(vertices, indices);
        
        // mesh.scale = Vec3::ONE * 250.0;
        mesh.pos += Vec3::ZERO.with_z(-4.0);
        
        self.meshes.push(mesh);
        
        self.renderer = Some(renderer);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::Resized(size) => {
                if let Some(renderer) = self.renderer.as_mut() {
                    renderer.resize(size.width, size.height);
                }
            }
            WindowEvent::RedrawRequested => {
                if let Some(renderer) = &self.renderer {
                    let time = Time {
                        total: self.start_time.elapsed().as_secs_f32(),
                        delta: self.start_time.elapsed().as_secs_f32(),
                    };
                    renderer.redraw(&self.meshes, time);
                }
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        let now = Instant::now();
        let vertecies = 1;
        let vertexes = 1;
        if let Some(renderer) = &self.renderer {
            if let Some(window) = self.window.as_ref() {
                window.request_redraw();
                let delta = now - self.last_time;
                let rotate = Quat::from_euler(EulerRot::XYZ, delta.as_secs_f32() * 1.89, 0.0, 0.0) * Quat::from_euler(EulerRot::XYZ, 0.0, delta.as_secs_f32() * 2.0, 0.0);
                self.meshes[0].rot = self.meshes[0].rot * rotate;
            }
        }
        self.last_time = now;
    }
}
