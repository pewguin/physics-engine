use crate::rendering::renderer::Renderer;
use anyhow::Result;
use futures::executor::block_on;
use std::sync::{Arc, RwLock};
use wgpu::hal::DynCommandEncoder;
use winit::application::ApplicationHandler;
use winit::dpi::{LogicalPosition, PhysicalSize, Position, Size};
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::monitor::VideoModeHandle;
use winit::window::{Fullscreen, Window, WindowAttributes, WindowId};

pub struct AppState {}
pub struct App {
    pub state: Arc<RwLock<AppState>>,
    pub renderer: Option<Renderer>, // pub physics: Arc<Physics>,
}
impl App {
    pub fn new() -> Self {
        Self {
            state: Arc::new(RwLock::new(AppState {})),
            renderer: None,
        }
    }
}
impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window_attributes = WindowAttributes::default();

        self.renderer = Some(Renderer::new(Arc::new(
            event_loop.create_window(window_attributes).unwrap(),
        )));
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
                if let Some(renderer) = &self.renderer {
                    renderer.redraw();
                }
            }
            WindowEvent::RedrawRequested => {
                if let Some(renderer) = &self.renderer {
                    renderer.redraw();
                }
            }
            _ => {}
        }
    }
}
