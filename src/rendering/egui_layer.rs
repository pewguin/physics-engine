use egui_wgpu::ScreenDescriptor;
use winit::{event::WindowEvent, window::Window};

use crate::debug::debug_panel;

pub struct EguiLayer {
    ctx: egui::Context,
    state: egui_winit::State,
    renderer: egui_wgpu::Renderer,
}

impl EguiLayer {
    pub fn new(device: &wgpu::Device, surface_format: wgpu::TextureFormat, window: &Window) -> Self {
        let ctx = egui::Context::default();
        let state = egui_winit::State::new(ctx.clone(), ctx.viewport_id(), window, None, None, None);
        let renderer = egui_wgpu::Renderer::new(device, surface_format, egui_wgpu::RendererOptions::default());
        Self { ctx, state, renderer }
    }

    pub fn on_window_event(&mut self, window: &Window, event: &WindowEvent) -> egui_winit::EventResponse {
        self.state.on_window_event(window, event)
    }

    pub fn draw(
        &mut self,
        window: &Window,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        size_px: [u32; 2],
    ) {
        let raw = self.state.take_egui_input(window);
        self.ctx.begin_pass(raw);
        debug_panel::ui(&self.ctx);
        let out = self.ctx.end_pass();
        self.state.handle_platform_output(window, out.platform_output);

        let jobs = self.ctx.tessellate(out.shapes, out.pixels_per_point);
        let screen = ScreenDescriptor { size_in_pixels: size_px, pixels_per_point: out.pixels_per_point };

        for (id, delta) in &out.textures_delta.set {
            self.renderer.update_texture(device, queue, *id, delta);
        }
        self.renderer.update_buffers(device, queue, encoder, &jobs, &screen);

        {
            let pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("egui"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: target,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations { load: wgpu::LoadOp::Load, store: wgpu::StoreOp::Store },
                })],
                ..Default::default()
            });
            self.renderer.render(&mut pass.forget_lifetime(), &jobs, &screen);
        }

        for id in &out.textures_delta.free {
            self.renderer.free_texture(id);
        }
    }
}
