use crate::physics::mesh::Mesh;
use crate::physics::world::World;
use crate::rendering::renderer::Renderer;
use crate::rendering::time::Time;
use crate::rendering::transform::Transform;
use glam::{EulerRot, Quat, Vec3};
use winit::keyboard::{KeyCode, PhysicalKey};
use std::f32::consts::PI;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use winit::application::ApplicationHandler;
use winit::event::{ElementState, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowAttributes, WindowId};
use crate::physics::collider::{ColliderBox, ColliderShape};
use crate::physics::rigid_body::RigidBody;
use crate::physics::spatial_grid::SpatialGrid;

const FRAME_TIME: Duration = Duration::from_nanos(16_666_667);

pub struct AppState {}
pub struct App<'a> {
    pub state: Arc<RwLock<AppState>>,
    pub renderer: Option<Renderer<'a>>,
    pub window: Option<Arc<Window>>,
    pub world: World,
    collision_grid: SpatialGrid,
    time: Time,
    last_time: Instant,
    start_time: Instant,
    accumulator: Duration,
}

impl App<'_> {
    pub fn new() -> Self {
        Self {
            state: Arc::new(RwLock::new(AppState {})),
            renderer: None,
            window: None,
            world: World::new(),
            collision_grid: SpatialGrid::new(2.0),
            time: Time {
                total: 0.0,
                delta: 0.0,
            },
            last_time: Instant::now(),
            start_time: Instant::now(),
            accumulator: Duration::ZERO,
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

        let box_collider = ColliderShape::ColliderBox(ColliderBox::new(Vec3::ZERO, Vec3::ONE, Quat::IDENTITY));

        let floor = Mesh::cube();
        let mut floor_transform = Transform::default();
        floor_transform.pos += Vec3::NEG_Y * 3.0;
        floor_transform.pos += Vec3::NEG_Z * 4.0;
        floor_transform.scale = Vec3::new(1.0, 0.1, 1.0) * 10.0;
        renderer.create_buffered_mesh(0, &floor);
        let rb = RigidBody::new(Vec3::ZERO, false);
        self.world.add_mesh(0, box_collider, floor_transform, floor, rb);
        self.collision_grid.add_object(0, true);

        let cube = Mesh::cube();
        let mut cube_transform = Transform::default();
        cube_transform.pos += Vec3::NEG_Z * 5.0;
        cube_transform.rot *= Quat::from_euler(EulerRot::XYZ, PI / 4.0, 0.0, PI / 4.0);
        renderer.create_buffered_mesh(1, &cube);
        let rb = RigidBody::new(Vec3::ZERO, true);
        self.world.add_mesh(1, box_collider, cube_transform, cube, rb);
        self.collision_grid.add_object(1, false);

        self.collision_grid.calculate_all_static_object_occupancies(&self.world);

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
            WindowEvent::KeyboardInput { event, .. } => {
                match (event.physical_key, event.state) {
                    (PhysicalKey::Code(KeyCode::Space), ElementState::Pressed) => {
                        // self.step_frame = true;
                    }
                    _ => {}
                }
            }
                
            _ => {}
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        if let Some(renderer) = &self.renderer {
            self.time = Time {
                total: self.start_time.elapsed().as_secs_f32(),
                delta: FRAME_TIME.as_secs_f32(),
            };
            self.accumulator += self.last_time.elapsed();
            while self.accumulator >= FRAME_TIME {
                self.accumulator -= FRAME_TIME;
                self.collision_grid.recalculate_grid_and_collisions(&mut self.world);
                self.world.do_physics_step(self.time.delta);
                renderer.redraw(&self.world, self.time);
            }
        }
        self.last_time = Instant::now();
    }
}
