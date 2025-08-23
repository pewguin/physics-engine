use crate::rendering::renderer::Renderer;
use crate::rendering::time::Time;
use crate::rendering::transform::Transform;
use std::sync::{Arc, RwLock};
use std::time::Instant;
use glam::{Quat, Vec3};
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowAttributes, WindowId};
use crate::physics::world::World;
use crate::rendering::vertex::Vertex;

pub struct AppState {}
pub struct App<'a> {
    pub state: Arc<RwLock<AppState>>,
    pub renderer: Option<Renderer<'a>>,
    pub window: Option<Arc<Window>>,
    pub world: World,
    last_time: Instant,
    start_time: Instant,
}
impl App<'_> {
    pub fn new() -> Self {
        Self {
            state: Arc::new(RwLock::new(AppState {})),
            renderer: None,
            window: None,
            world: World::new(),
            last_time: Instant::now(),
            start_time: Instant::now(),
        }
    }
}
impl ApplicationHandler for App<'_> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window_attributes = WindowAttributes::default();//.with_name("renderer", "of things");
        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());
        self.window = Some(window.clone());
        let mut renderer = Renderer::new(
            window
        );

        let vertices = vec![
            Vertex { position: [-1.0, -1.0,  1.0], uv: [0.0, 0.0] },
            Vertex { position: [ 1.0, -1.0,  1.0], uv: [1.0, 0.0] },
            Vertex { position: [ 1.0,  1.0,  1.0], uv: [1.0, 1.0] },
            Vertex { position: [-1.0,  1.0,  1.0], uv: [0.0, 1.0] },

            Vertex { position: [ 1.0, -1.0, -1.0], uv: [0.0, 0.0] },
            Vertex { position: [-1.0, -1.0, -1.0], uv: [1.0, 0.0] },
            Vertex { position: [-1.0,  1.0, -1.0], uv: [1.0, 1.0] },
            Vertex { position: [ 1.0,  1.0, -1.0], uv: [0.0, 1.0] },

            Vertex { position: [-1.0, -1.0, -1.0], uv: [0.0, 0.0] },
            Vertex { position: [-1.0, -1.0,  1.0], uv: [1.0, 0.0] },
            Vertex { position: [-1.0,  1.0,  1.0], uv: [1.0, 1.0] },
            Vertex { position: [-1.0,  1.0, -1.0], uv: [0.0, 1.0] },

            Vertex { position: [ 1.0, -1.0,  1.0], uv: [0.0, 0.0] },
            Vertex { position: [ 1.0, -1.0, -1.0], uv: [1.0, 0.0] },
            Vertex { position: [ 1.0,  1.0, -1.0], uv: [1.0, 1.0] },
            Vertex { position: [ 1.0,  1.0,  1.0], uv: [0.0, 1.0] },

            Vertex { position: [-1.0,  1.0,  1.0], uv: [0.0, 0.0] },
            Vertex { position: [ 1.0,  1.0,  1.0], uv: [1.0, 0.0] },
            Vertex { position: [ 1.0,  1.0, -1.0], uv: [1.0, 1.0] },
            Vertex { position: [-1.0,  1.0, -1.0], uv: [0.0, 1.0] },

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

        let mesh = renderer.create_mesh(vertices, indices);
        let mut transform = Transform::default();
        transform.pos += Vec3::NEG_Z * 4.0;
        self.world.add_mesh(0, transform, Vec3::ZERO, mesh);
                
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
                    renderer.redraw(&self.world, time);
                }
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        let now = Instant::now();
        if let Some(renderer) = &self.renderer {
            if let Some(window) = self.window.as_ref() {
                let run_duration = (now - self.start_time);
                self.world.transforms.get_mut(&0).unwrap().rot = Quat::from_euler(glam::EulerRot::XYZ, 0.0, run_duration.as_secs_f32(), 0.0);
                self.world.transforms.get_mut(&0).unwrap().pos = (Vec3::Y * run_duration.as_secs_f32().sin() * 2.0) + Vec3::NEG_Z * 4.0;
                window.request_redraw();
            }
        }
        self.last_time = now;
    }
}
