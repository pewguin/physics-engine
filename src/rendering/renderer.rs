use std::borrow::Cow;
use futures::executor::block_on;
use std::sync::Arc;
use glam::{Mat4, Quat, Vec3};
use wgpu::{Adapter, AddressMode, Backends, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingResource, BindingType, Buffer, BufferBinding, BufferBindingType, BufferUsages, Color, ColorTargetState, CompareFunction, DepthStencilState, Device, DeviceDescriptor, Extent3d, FilterMode, FragmentState, IndexFormat, Instance, InstanceDescriptor, InstanceFlags, Label, LoadOp, Operations, Origin2d, Origin3d, PipelineLayout, PipelineLayoutDescriptor, PowerPreference, PresentMode, PrimitiveState, Queue, RenderPassColorAttachment, RenderPassDepthStencilAttachment, RenderPassDescriptor, RenderPipeline, RenderPipelineDescriptor, RequestAdapterOptions, Sampler, SamplerBindingType, ShaderModuleDescriptor, ShaderSource, ShaderStages, StoreOp, Surface, SurfaceConfiguration, TexelCopyBufferLayout, TexelCopyTextureInfo, Texture, TextureAspect, TextureDimension, TextureFormat, TextureSampleType, TextureUsages, TextureView, TextureViewDimension, VertexBufferLayout, VertexState};
use wgpu::Face::Back;
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::wgt::{BufferDescriptor, SamplerDescriptor, TextureDescriptor, TextureViewDescriptor};
use winit::dpi::Size;
use winit::window::Window;
use crate::rendering::mesh::Mesh;
use crate::rendering::time::Time;
use crate::rendering::vertex::Vertex;

pub struct Renderer<'a> {
    surface: Surface<'a>,
    device: Device,
    queue: Queue,
    pipeline: RenderPipeline,
    mesh_bind_group_layout: BindGroupLayout,
    projection_matrix: Mat4,
    time_buffer: Buffer,
    global_bind_group: BindGroup,
}

impl Renderer<'_> {
    pub fn new(window: Arc<Window>) -> Self {
        let wgpu = Instance::new(&InstanceDescriptor {
            backends: Backends::GL,
            flags: InstanceFlags::from_env_or_default(),
            memory_budget_thresholds: Default::default(),
            backend_options: Default::default(),
        });

        let surface = wgpu.create_surface(window).unwrap();

        let (device, queue) = block_on(
            block_on(wgpu.request_adapter(&RequestAdapterOptions {
                power_preference: PowerPreference::None,
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            }))
                .unwrap()
                .request_device(&DeviceDescriptor {
                    label: Label::from("main"),
                    required_features: Default::default(),
                    required_limits: Default::default(),
                    memory_hints: Default::default(),
                    trace: Default::default(),
                }),
        ).unwrap();

        surface.configure(
            &device,
            &SurfaceConfiguration {
                usage: TextureUsages::RENDER_ATTACHMENT,
                format: TextureFormat::Rgba16Float,
                width: 300,
                height: 300,
                present_mode: PresentMode::Fifo,
                desired_maximum_frame_latency: 2,
                alpha_mode: Default::default(),
                view_formats: vec![TextureFormat::Rgba16Float],
            },
        );
        
        let shader_module = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("main"),
            source: ShaderSource::Wgsl(Cow::Borrowed(include_str!("shader.wgsl"))),
        });

        let mesh_bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Label::from("Mesh Uniform"),
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::VERTEX,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let global_bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Label::from("Global bindings"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT | ShaderStages::VERTEX,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float {
                            filterable: true,
                        },
                        view_dimension: TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                }
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Label::from("Pipeline layout"),
            bind_group_layouts: &[&mesh_bind_group_layout, &global_bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Main pipeline"),
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &shader_module,
                entry_point: None,
                compilation_options: Default::default(),
                buffers: &[Vertex::LAYOUT],
            },
            primitive: PrimitiveState {
                cull_mode: Some(Back),
                ..Default::default()
            },
            depth_stencil: None,
            multisample: Default::default(),
            fragment: Some(FragmentState {
                module: &shader_module,
                entry_point: None,
                compilation_options: Default::default(),
                targets: &[Some(ColorTargetState {
                    format: TextureFormat::Rgba16Float,
                    blend: None,
                    write_mask: Default::default(),
                })],
            }),
            multiview: None,
            cache: None,
        });

        let projection_matrix = Mat4::perspective_rh_gl(
            90.0_f32.to_radians(),
            1.0,
            0.0001,
            3000.0,
        );

        let time = Time {
            total: 0.0,
            delta: 0.0,
        };

        let time_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Label::from("Time buffer"),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            contents: &bytemuck::cast_slice(&[time]),
        });

        let (obama_tex, obama_view, obama_sampler) = Self::load_texture(&device, &queue, "assets/obama.webp");

        let global_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Label::from("Global bind group"),
            layout: &global_bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: time_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::TextureView(&obama_view),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: BindingResource::Sampler(&obama_sampler),
                }
            ],
        });
        Renderer { 
            surface,
            device,
            queue,
            pipeline,
            mesh_bind_group_layout,
            projection_matrix,
            time_buffer,
            global_bind_group,
        }
    }
    pub fn resize(&mut self, width: u32, height: u32) {
        self.surface.configure(
            &self.device,
            &SurfaceConfiguration {
                usage: TextureUsages::RENDER_ATTACHMENT,
                format: TextureFormat::Rgba16Float,
                width,
                height,
                present_mode: PresentMode::Fifo,
                desired_maximum_frame_latency: 2,
                alpha_mode: Default::default(),
                view_formats: vec![TextureFormat::Rgba16Float],
            }
        );
        self.projection_matrix = Mat4::orthographic_rh_gl(
            -(width as f32) / 2.0, width as f32 / 2.0,
            -(height as f32) / 2.0, height as f32 / 2.0,
            -1.0, 1.0
        );
        self.projection_matrix = Mat4::perspective_rh_gl(
            90.0_f32.to_radians(),
            width as f32 / height as f32,
            0.0001,
            3000.0,
        );
    }
    pub fn redraw(&self, meshes: &Vec<Mesh>, time: Time) {
        let tex = self.surface.get_current_texture().unwrap();
        let view = tex.texture.create_view(&Default::default());
        let mut encoder = self.device.create_command_encoder(&Default::default());

        let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("pass 1"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: &view,
                depth_slice: None,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Clear(Color {
                        r: 0.0,
                        g: 0.0,
                        b: 0.0,
                        a: 1.0,
                    }),
                    store: StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        render_pass.set_pipeline(&self.pipeline);
        self.queue.write_buffer(
            &self.time_buffer, 0,
            bytemuck::cast_slice(&[time]),
        );
        render_pass.set_bind_group(1, &self.global_bind_group, &[]);

        for mesh in meshes {
            self.queue.write_buffer(
                &mesh.uniform_buffer, 0, 
                bytemuck::cast_slice(&(self.projection_matrix * mesh.get_transform()).to_cols_array())
            );
            render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
            render_pass.set_bind_group(0, &mesh.bind_group, &[]);
            render_pass.set_index_buffer(mesh.index_buffer.slice(..), IndexFormat::Uint16);
            render_pass.draw_indexed(0..mesh.index_count, 0, 0..1);
        }
        drop(render_pass);

        self.queue.submit(Some(encoder.finish()));
        tex.present();
    }

    pub fn create_mesh(&mut self, verts: Vec<Vertex>, indices: Vec<u16>) -> Mesh {
        let uniform_buffer = self.device.create_buffer_init(&BufferInitDescriptor {
            label: Label::from("Uniform Buffer"),
            contents: &bytemuck::cast_slice(&self.projection_matrix.to_cols_array()),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });
        Mesh {
            vertex_buffer: self.device.create_buffer_init(&BufferInitDescriptor {
                label: Label::from("Vertex Buffer"),
                contents: &bytemuck::cast_slice(&verts),
                usage: BufferUsages::VERTEX,
            }),
            vertex_count: verts.len() as u32,
            index_buffer: self.device.create_buffer_init(&BufferInitDescriptor {
                label: Label::from("Index Buffer"),
                contents: &bytemuck::cast_slice(&indices),
                usage: BufferUsages::INDEX,
            }),
            index_count: indices.len() as u32,
            bind_group: self.device.create_bind_group(&BindGroupDescriptor {
                label: Label::from("Mesh Uniform Bind Group"),
                layout: &self.mesh_bind_group_layout,
                entries: &[BindGroupEntry {
                    binding: 0,
                    resource: uniform_buffer.as_entire_binding(),
                }],
            }),
            uniform_buffer,
            pos: Vec3::ZERO,
            rot: Quat::IDENTITY,
            scale: Vec3::ONE
        }
    }
    pub fn load_texture(device: &Device, queue: &Queue, path: &str) -> (Texture, TextureView, Sampler) {
        let img = image::open(path).unwrap();
        let rgba = img.to_rgba8();
        let (width, height) = rgba.dimensions();
        let size = Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };
        let texture = device.create_texture(&TextureDescriptor {
            label: Some("Obama"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8UnormSrgb,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
            view_formats: &[],
        });
        queue.write_texture(
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
        let sampler = device.create_sampler(&SamplerDescriptor {
            label: Some("Obama Sampler"),
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
}
