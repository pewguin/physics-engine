use crate::physics::mesh::Mesh;
use crate::physics::world::World;
use crate::rendering::renderer::Renderer;
use crate::rendering::time::Time;
use crate::rendering::transform::Transform;
use glam::{EulerRot, Quat, Vec3};
use winit::keyboard::{KeyCode, PhysicalKey};
use std::f32::consts::PI;
use std::sync::{Arc, RwLock};
use std::thread::sleep;
use std::time::{Duration, Instant};
use winit::application::ApplicationHandler;
use winit::event::{ElementState, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowAttributes, WindowId};
use crate::physics;
use crate::physics::collider::{Box3, Sphere};
use crate::physics::rigid_body::RigidBody;
use crate::physics::spatial_grid::SpatialGrid;

// Amount of discrete physics steps per render step
// Larger values are more precise, but take longer to process
// 0 will disable physics
const PHYSICS_SUBSTEPS_COUNT: u16 = 256;
// Time between render steps
// 16,666,667 nanos is about 1/60th of a second
const FRAME_TIME: Duration = Duration::from_nanos(16_666_667);

pub struct App<'a> {
    pub renderer: Option<Renderer<'a>>,
    pub window: Option<Arc<Window>>,
    pub world: World,
    collision_grid: SpatialGrid,
    total_time: f32,
    last_time: Instant,
    time_accumulator: Duration,
}

impl App<'_> {
    pub fn new() -> Self {
        Self {
            renderer: None,
            window: None,
            world: World::new(),
            collision_grid: SpatialGrid::new(2.0),
            total_time: 0.0,
            last_time: Instant::now(),
            time_accumulator: Duration::ZERO,
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

        let box_collider = Box3 {
            center: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            half_extents: Vec3::splat(0.5),
        };
        
        // Create static floor
        let floor = Mesh::cube();
        let mut floor_transform = Transform::default();
        floor_transform.pos = Vec3::NEG_Y * 3.0;
        floor_transform.pos += Vec3::NEG_Z * 4.0;
        floor_transform.scale = Vec3::new(1.0, 0.1, 1.0) * 10.0;
        renderer.create_buffered_mesh(0, &floor);
        self.world.add_mesh(0, box_collider, floor_transform, floor);
        self.collision_grid.add_object(0, true);
        
        // Create physics-influenced cube
        let cube = Mesh::cube();
        let mut cube_transform = Transform::default();
        cube_transform.pos = Vec3::NEG_Z * 5.0;
        cube_transform.rot *= Quat::from_euler(EulerRot::XYZ, PI / 4.0, 0.0, PI / 4.0);
        renderer.create_buffered_mesh(1, &cube);
        let cube_rb = RigidBody::new(Vec3::ZERO, true, 0.98);
        self.world.add_object(1, box_collider, cube_transform, cube, cube_rb);
        self.collision_grid.add_object(1, false);

        let sphere_collider = Sphere {
            center: Vec3::ZERO,
            radius: 0.5,
        };
        
        // Create physics influenced sphere
        let sphere_rb = RigidBody::new(Vec3::ZERO, true, 0.98);
        let sphere = Mesh::cube();
        let mut sphere_transform = Transform::default();
        sphere_transform.pos = Vec3::new(0.0, 3.0, -5.0);
        renderer.create_buffered_mesh(2, &sphere);
        self.world.add_object(2, sphere_collider, sphere_transform, sphere, sphere_rb);
        self.collision_grid.add_object(2, false);

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
        let now = Instant::now();
        let frame_dt = now - self.last_time;
        self.last_time = now;

        self.time_accumulator += frame_dt;
        
        // Physics loop, runs until its caught up with all the physics it must do
        while self.time_accumulator >= FRAME_TIME {
            let frame_start = Instant::now();
            self.time_accumulator -= FRAME_TIME;

            let time = Time {
                total: self.total_time,
                delta: FRAME_TIME.as_secs_f32(),
            };
            
            // Split physics up to make it less discrete
            for _ in 0..PHYSICS_SUBSTEPS_COUNT {
                self.collision_grid.recalculate_grid_and_collisions(&mut self.world);
                self.world.do_physics_step(time.delta / PHYSICS_SUBSTEPS_COUNT as f32);
            }

            self.total_time += FRAME_TIME.as_secs_f32();

            if frame_start.elapsed() > FRAME_TIME {
                // If loop consistently overruns, it may be time for some optimization
                println!("Loop overrun of {}ms", (frame_start.elapsed() - FRAME_TIME).as_millis());
            }
        }

        // Render after physics is all done
        if let Some(renderer) = &self.renderer {
            renderer.redraw(&self.world);
        }
    }
}
