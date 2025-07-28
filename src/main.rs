use crate::app::App;
use crate::rendering::renderer::Renderer;
use log::error;
use wgpu::Surface;
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::Window;

mod app;
mod rendering;

fn main() {
    env_logger::init();

    let event_loop = EventLoop::new().expect("Failed to create event loop");
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::new();
    event_loop.run_app(&mut app).unwrap();
}