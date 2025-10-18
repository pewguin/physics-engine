use glam::Vec3;
use crate::app::App;
use winit::event_loop::{ControlFlow, EventLoop};
use crate::physics::collider::{ColliderShape, Sphere};

mod app;
mod rendering;
mod physics;

fn main() {
    let sphere_a = Box::new(Sphere {
        radius: 1.0,
        center: Vec3::ZERO,
    });
    let sphere_b  = Box::new(Sphere {
        radius: 1.0,
        center: Vec3::X,
    });

    env_logger::init();

    let event_loop = EventLoop::new().expect("Failed to create event loop");
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::new();
    event_loop.run_app(&mut app).unwrap();
}