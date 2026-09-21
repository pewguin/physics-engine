use crate::physics::mesh::Mesh;
use crate::rendering::buffered_mesh::BufferedMesh;
use crate::rendering::camera::Camera;
use crate::rendering::egui_layer::EguiLayer;
use crate::rendering::frame::Frame;
use crate::rendering::material::Material;
use crate::rendering::renderers::mesh_renderer::MeshRenderer;
use crate::rendering::renderers::wireframe_renderer::{self, WireframeRenderer};
use crate::rendering::transform::Transform;
use crate::rendering::buffered_wireframe_mesh::BufferedWireframeMesh;
use futures::executor::block_on;
use glam::{Mat4, Quat, UVec2, Vec3};
use winit::event::WindowEvent;
use std::rc::Rc;
use std::sync::Arc;
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::wgt::{SamplerDescriptor, TextureDescriptor, TextureViewDescriptor};
use wgpu::{AddressMode, Backends, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingResource, BindingType, Buffer, BufferBindingType, BufferUsages, Color, ColorTargetState, CompareFunction, DepthStencilState, Device, DeviceDescriptor, ExperimentalFeatures, Extent3d, FilterMode, FragmentState, IndexFormat, Instance, InstanceDescriptor, InstanceFlags, Label, LoadOp, Operations, Origin3d, PipelineLayoutDescriptor, PowerPreference, PresentMode, PrimitiveState, Queue, RenderPass, RenderPassColorAttachment, RenderPassDepthStencilAttachment, RenderPassDescriptor, RenderPipeline, RenderPipelineDescriptor, RequestAdapterOptions, Sampler, SamplerBindingType, ShaderModuleDescriptor, ShaderSource, ShaderStages, StoreOp, Surface, SurfaceConfiguration, TexelCopyBufferLayout, TexelCopyTextureInfo, Texture, TextureAspect, TextureDimension, TextureFormat, TextureSampleType, TextureUsages, TextureView, TextureViewDimension, VertexState};
use winit::window::Window;

const TEXTURE_FORMAT: TextureFormat = TextureFormat::Rgba16Float;

pub struct Renderer<'a> {
    // Control
    surface: Surface<'a>,
    device: Device,
    queue: Queue,

    // Frame buffers
    projection_buffer: Buffer,
    camera_transform_buffer: Buffer,
    viewport_buffer: Buffer,
    frame_bind_group: BindGroup,

    // Renderers
    mesh_renderer: MeshRenderer,
    wireframe_renderer: WireframeRenderer,

    // Camera
    pub camera: Camera,

    // Depth texture
    depth_texture: Texture,
    depth_texture_view: TextureView,

    // Egui
    egui_layer: EguiLayer,
}

impl Renderer<'_> {
    /// Creates a new renderer with a target surface.
    /// # Arguments
    ///
    /// * `window`:
    /// Window to render to
    ///
    /// returns: Renderer
    pub fn new(window: Arc<Window>) -> Self {
        let wgpu = Instance::new(&InstanceDescriptor {
            backends: Backends::PRIMARY,
            flags: InstanceFlags::from_env_or_default(),
            memory_budget_thresholds: Default::default(),
            backend_options: Default::default(),
        });

        let surface = wgpu.create_surface(window.clone()).unwrap();
    
        let adapter = block_on(wgpu.request_adapter(&RequestAdapterOptions {
            power_preference: PowerPreference::None,
            force_fallback_adapter: false,
            compatible_surface: Some(&surface),
        })).unwrap();

        let (device, queue) = block_on(
                adapter.request_device(&DeviceDescriptor {
                    label: Label::from("main"),
                    required_features: Default::default(),
                    required_limits: Default::default(),
                    memory_hints: Default::default(),
                    trace: Default::default(),
                    experimental_features: ExperimentalFeatures::disabled(),
                }),
        ).unwrap();

        let surface_inital_size = UVec2::new(300, 300);
        let surface_configuration = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: TEXTURE_FORMAT,
            width: surface_inital_size.x,
            height: surface_inital_size.y,
            present_mode: PresentMode::Fifo,
            desired_maximum_frame_latency: 2,
            alpha_mode: Default::default(),
            view_formats: vec![TEXTURE_FORMAT],
        };

        surface.configure(
            &device,
            &surface_configuration,
        );
        
        let camera = Camera::new();

        // Stuff that stays the same throughout each frame
        let frame_bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Label::from("Global bindings"),
            entries: &[
                // Projection matrix
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // Camera transform
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::VERTEX,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // Viewport (size of the viewport)
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStages::VERTEX,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let projection_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Label::from("Projection buffer"),
            contents: &bytemuck::cast_slice(&camera.projection_matrix.to_cols_array()),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });
        let camera_transform_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Label::from("View buffer"),
            contents: &bytemuck::cast_slice(&camera.transform.as_matrix().inverse().to_cols_array()),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });
        let viewport_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Label::from("Viewport buffer"),
            contents: &bytemuck::bytes_of(&[surface_inital_size.x as f32, surface_inital_size.y as f32]),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        // Bindings for all meshes each frame
        let frame_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Label::from("frame bind group"),
            layout: &frame_bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: projection_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: camera_transform_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: viewport_buffer.as_entire_binding(),
                }
            ],
        });

        let mesh_renderer = MeshRenderer::new(&device, &frame_bind_group_layout);
        let wireframe_renderer = WireframeRenderer::new(&device, &frame_bind_group_layout);

        let (depth_texture, depth_texture_view) = Self::create_depth_texture(&device, surface_inital_size.x, surface_inital_size.y);

        let surface = wgpu.create_surface(window.clone()).unwrap();
        let egui_layer = EguiLayer::new(&device, TEXTURE_FORMAT, &window);

        Renderer { 
            // Control
            surface,
            device,
            queue,

            // Buffers
            projection_buffer,
            camera_transform_buffer,
            viewport_buffer,
            frame_bind_group,

            // Renderers
            mesh_renderer,
            wireframe_renderer,

            // Camera
            camera,

            // Depth texture 
            depth_texture,
            depth_texture_view,

            // egui
            egui_layer
        }
    }

    // Resize all render targets and proj. matrix
    pub fn resize(&mut self, width: u32, height: u32) {
        self.surface.configure(
            &self.device,
            &SurfaceConfiguration {
                usage: TextureUsages::RENDER_ATTACHMENT,
                format: TEXTURE_FORMAT,
                width,
                height,
                present_mode: PresentMode::Fifo,
                desired_maximum_frame_latency: 2,
                alpha_mode: Default::default(),
                view_formats: vec![TEXTURE_FORMAT],
            }
        );
        self.camera.update_proj_matrix(width as f32, height as f32);

        self.queue.write_buffer(&self.projection_buffer, 0, bytemuck::cast_slice(&self.camera.projection_matrix.to_cols_array()));
        self.queue.write_buffer(&self.viewport_buffer, 0, bytemuck::bytes_of(&[width as f32, height as f32]));
        
        (self.depth_texture, self.depth_texture_view) = Self::create_depth_texture(&self.device, width, height);
    }

    pub fn begin_frame(&self) -> Frame {
        let surface_texture = self.surface.get_current_texture().unwrap();
        let view = surface_texture.texture.create_view(&Default::default());
        let encoder = self.device.create_command_encoder(&Default::default());

        self.queue.write_buffer(
            &self.camera_transform_buffer, 0,
            bytemuck::cast_slice(&self.camera.transform.as_matrix().inverse().to_cols_array()),
        );

        Frame {
            surface_texture,
            view,
            encoder,
        }
    }

    pub fn begin_pass<'a>(&self, frame: &'a mut Frame) -> RenderPass<'a> {
        frame.encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("pass 1"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: &frame.view,
                depth_slice: None,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Clear(Color::BLACK),
                    store: StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                view: &self.depth_texture_view,
                depth_ops: Some(Operations {
                    load: LoadOp::Clear(1.0),
                    store: StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
        })
    }

    pub fn draw_mesh(&self, render_pass: &mut RenderPass, mesh: &BufferedMesh) {
        self.mesh_renderer.draw(&self.frame_bind_group, render_pass, mesh);
    }

    pub fn draw_wireframe(&self, render_pass: &mut RenderPass, mesh: &BufferedWireframeMesh) {
        self.wireframe_renderer.draw(&self.frame_bind_group, render_pass, mesh);
    }

    pub fn present_frame(&self, frame: Frame) {
        self.queue.submit(Some(frame.encoder.finish()));
        frame.surface_texture.present();
    }

    pub fn upload_mesh(&self, mesh: &Mesh, transform: &Transform, material: Rc<Material>) -> BufferedMesh {
        self.mesh_renderer.upload_mesh(&self.device, mesh, transform, material)
    }

    pub fn update_mesh(&self, mesh: &BufferedMesh, transform: &Transform) {
        self.queue.write_buffer(&mesh.transform_buffer, 0, bytemuck::bytes_of(&transform.as_matrix().to_cols_array()));
    }

    pub fn upload_wireframe(&self, mesh: &Mesh, transform: &Transform) -> BufferedWireframeMesh {
        self.wireframe_renderer.upload_mesh(&self.device, mesh, transform)
    }

    pub fn upload_wireframe_verts(&self, verts: &[Vec3]) -> BufferedWireframeMesh {
        self.wireframe_renderer.upload_verts(&self.device, verts)
    }

    pub fn update_wireframe(&self, mesh: &BufferedWireframeMesh, transform: &Transform) {
        self.queue.write_buffer(&mesh.transform_buffer, 0, bytemuck::bytes_of(&transform.as_matrix().to_cols_array()));
    }

    pub fn egui_event(&mut self, window: &Window, event: &WindowEvent) -> egui_winit::EventResponse {
        self.egui_layer.on_window_event(window, event)
    }

    pub fn draw_debug_ui(&mut self, window: &Window, frame: &mut Frame) {
        let tex = &frame.surface_texture.texture;
        let size = [tex.width(), tex.height()];
        self.egui_layer.draw(window, &self.device, &self.queue, &mut frame.encoder, &frame.view, size);
    }

    pub fn create_material(&self, path: &str) -> Material {
        let (texture, texture_view, sampler) = self.load_texture(path);
        let bind_group = self.device.create_bind_group(&BindGroupDescriptor { 
            label: Label::from(format!("{} material bind group", path).as_str()),
            layout: &self.mesh_renderer.material_bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(&texture_view),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Sampler(&sampler),
                },
            ],
        });

        Material {
            texture,
            sampler,
            bind_group
        }
    }
    

    pub fn load_texture(&self, path: &str) -> (Texture, TextureView, Sampler) {
        let img = image::open(path).unwrap();
        let rgba = img.to_rgba8();
        let (width, height) = rgba.dimensions();
        let size = Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };
        let texture = self.device.create_texture(&TextureDescriptor {
            label: Some(path),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8UnormSrgb,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
            view_formats: &[],
        });
        self.queue.write_texture(
            TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: Origin3d::ZERO,
                aspect: TextureAspect::All,
            },
            &rgba,
            TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * width),
                rows_per_image: Some(height),
            },
            size
        );
        let view = texture.create_view(&TextureViewDescriptor::default());
        let sampler = self.device.create_sampler(&SamplerDescriptor {
            label: Some(&format!("{} Sampler", path)),
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Linear,
            mipmap_filter: FilterMode::Nearest,
            ..Default::default()
        });

        (texture, view, sampler)
    }

    pub fn create_depth_texture(device: &Device, width: u32, height: u32) -> (Texture, TextureView) {
        let depth_texture = device.create_texture(&TextureDescriptor {
            label: Some("depth texture"),
            size: Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Depth32Float,
            usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let depth_texture_view = depth_texture.create_view(&TextureViewDescriptor::default());

        (depth_texture, depth_texture_view)
    }
}
