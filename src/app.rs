use std::f32::consts::PI;
use crate::rendering::renderer::Renderer;
use crate::rendering::time::Time;
use crate::rendering::transform::Transform;
use std::sync::{Arc, RwLock};
use std::time::Instant;
use glam::{EulerRot, Quat, Vec3};
use wgpu::VertexStepMode::Instance;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowAttributes, WindowId};
use crate::physics::world::World;
use crate::physics::mesh::Mesh;
use crate::rendering::vertex::Vertex;

pub struct AppState {}
pub struct App<'a> {
    pub state: Arc<RwLock<AppState>>,
    pub renderer: Option<Renderer<'a>>,
    pub window: Option<Arc<Window>>,
    pub world: World,
    time: Time,
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
            time: Time {
                total: 0.0,
                delta: 0.0,
            },
            last_time: Instant::now(),
            start_time: Instant::now(),
        }
    }
}
impl ApplicationHandler for App<'_> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window_attributes = WindowAttributes::default();
        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());
        self.window = Some(window.clone());
        let mut renderer = Renderer::new(
            window
        );

        let floor = Mesh::cube();
        let mut floor_transform = Transform::default();
        floor_transform.pos += Vec3::NEG_Y * 3.0;
        floor_transform.scale = Vec3::new(1.0, 0.1, 1.0) * 10.0;
        renderer.create_buffered_mesh(0, &floor);
        self.world.add_mesh(0, floor_transform, Vec3::ZERO, floor);

        let cube = Mesh::cube();
        let mut cube_transform = Transform::default();
        cube_transform.pos += Vec3::NEG_Z * 3.0;
        cube_transform.rot *= Quat::from_euler(EulerRot::XYZ, PI / 4.0, 0.0, PI / 4.0);
        renderer.create_buffered_mesh(1, &cube);
        self.world.add_mesh(1, cube_transform, Vec3::ZERO, cube);
                
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
                    renderer.redraw(&self.world, self.time);
                }
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        self.time = Time {
            total: self.start_time.elapsed().as_secs_f32(),
            delta: self.last_time.elapsed().as_secs_f32(),
        };
        if let Some(renderer) = &self.renderer {
            if let Some(window) = self.window.as_ref() {
                let pos_ref = match self.world.transforms.get_mut(&1) {
                    Some(transform) => &mut transform.pos,
                    None => panic!(),
                };
                *pos_ref += Vec3::NEG_Y * 0.1 * self.time.delta;
            }
            renderer.redraw(&self.world, self.time);
        }
    }
}
