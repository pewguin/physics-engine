use glam::{Quat, Vec3};
use crate::app::App;
use winit::event_loop::{ControlFlow, EventLoop};
use crate::physics::collider::{Box3, ColliderShape, Sphere};

mod app;
mod rendering;
mod physics;
mod debug;

fn main() {
    env_logger::init();

    let event_loop = EventLoop::new().expect("Failed to create event loop");
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::new();
    event_loop.run_app(&mut app).unwrap();
}
