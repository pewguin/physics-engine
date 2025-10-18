use crate::physics::mesh::Mesh;
use crate::physics::world::World;
use crate::rendering::buffered_mesh::BufferedMesh;
use crate::rendering::time::Time;
use crate::rendering::transform::Transform;
use crate::rendering::vertex::Vertex;
use futures::executor::block_on;
use glam::{Mat4, Quat, Vec3};
use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::Arc;
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::wgt::{SamplerDescriptor, TextureDescriptor, TextureViewDescriptor};
use wgpu::Face::Back;
use wgpu::{AddressMode, Backends, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingResource, BindingType, Buffer, BufferBindingType, BufferUsages, Color, ColorTargetState, CompareFunction, DepthStencilState, Device, DeviceDescriptor, Extent3d, FilterMode, FragmentState, IndexFormat, Instance, InstanceDescriptor, InstanceFlags, Label, LoadOp, Operations, Origin3d, PipelineLayoutDescriptor, PowerPreference, PresentMode, PrimitiveState, Queue, RenderPassColorAttachment, RenderPassDepthStencilAttachment, RenderPassDescriptor, RenderPipeline, RenderPipelineDescriptor, RequestAdapterOptions, Sampler, SamplerBindingType, ShaderModuleDescriptor, ShaderSource, ShaderStages, StoreOp, Surface, SurfaceConfiguration, TexelCopyBufferLayout, TexelCopyTextureInfo, Texture, TextureAspect, TextureDimension, TextureFormat, TextureSampleType, TextureUsages, TextureView, TextureViewDimension, VertexState};
use winit::window::Window;

pub struct Renderer<'a> {
    surface: Surface<'a>,
    device: Device,
    queue: Queue,
    pipeline: RenderPipeline,
    mesh_bind_group_layout: BindGroupLayout,
    projection_matrix: Mat4,
    time_buffer: Buffer,
    projection_buffer: Buffer,
    view_buffer: Buffer,
    frame_bind_group: BindGroup,
    texture_bind_group: BindGroup,
    depth_texture: Texture,
    depth_texture_view: TextureView,
    camera_transform: Transform,
    buffered_meshes: HashMap<u32, BufferedMesh>,
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

        let surface_configuration = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: TextureFormat::Rgba16Float,
            width: 300,
            height: 300,
            present_mode: PresentMode::Fifo,
            desired_maximum_frame_latency: 2,
            alpha_mode: Default::default(),
            view_formats: vec![TextureFormat::Rgba16Float],
        };

        surface.configure(
            &device,
            &surface_configuration,
        );

        // Prepare shader code
        let shader_module = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("main"),
            source: ShaderSource::Wgsl(Cow::Borrowed(include_str!("shader.wgsl"))),
        });

        // Layout for data passed to shader for each mesh. Updated per mesh.
        let mesh_bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Label::from("Mesh bindings layout"),
            entries: &[
                // Model matrix
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
            ],
        });

        // Layout for data passed to all meshes. Updated per frame.
        let frame_bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Label::from("Global bindings"),
            entries: &[
                // Time binding
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
                // View
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
                // Project
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

        // Layout for texture data passed to shader. Updated rarely.
        let texture_bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("texture bind group layout"),
            entries: &[
                // Texture
                BindGroupLayoutEntry {
                    binding: 0,
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
                // Texture sampler
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Label::from("Pipeline layout"),
            bind_group_layouts: &[&texture_bind_group_layout, &frame_bind_group_layout, &mesh_bind_group_layout,],
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
            depth_stencil: Some(DepthStencilState {
                format: TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: CompareFunction::LessEqual,
                stencil: Default::default(),
                bias: Default::default(),
            }),
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

        let camera_transform = Transform {
            scale: Vec3::ONE,
            rot: Quat::IDENTITY,
            pos: Vec3::ZERO,
        };

        let time_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Label::from("Time buffer"),
            contents: &bytemuck::cast_slice(&[time]),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });
        let projection_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Label::from("Projection buffer"),
            contents: &bytemuck::cast_slice(&projection_matrix.to_cols_array()),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });
        let view_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Label::from("View buffer"),
            contents: &bytemuck::cast_slice(&camera_transform.as_matrix().to_cols_array()),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        // Bindings for all meshes each frame
        let frame_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Label::from("frame bind group"),
            layout: &frame_bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: time_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: projection_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: view_buffer.as_entire_binding(),
                }
            ],
        });

        let (_, obama_view, obama_sampler) = Self::load_texture(&device, &queue, "assets/obama.webp");

        let texture_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Label::from("Texture bind group"),
            layout: &texture_bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(&obama_view)
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Sampler(&obama_sampler)
                }
            ],
        });

        let (depth_texture, depth_texture_view) = Self::create_depth_texture(&device, surface_configuration.width, surface_configuration.height);

        let buffered_meshes = HashMap::<u32, BufferedMesh>::new();
        Renderer { 
            surface,
            device,
            queue,
            pipeline,
            mesh_bind_group_layout,
            projection_matrix,
            time_buffer,
            frame_bind_group,
            depth_texture,
            depth_texture_view,
            camera_transform,
            projection_buffer,
            view_buffer,
            texture_bind_group,
            buffered_meshes,
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
        self.projection_matrix = Mat4::perspective_rh_gl(
            90.0_f32.to_radians(),
            width as f32 / height as f32,
            0.0001,
            3000.0,
        );
        let (depth_texture, depth_texture_view) = Self::create_depth_texture(&self.device, width, height);
        self.depth_texture = depth_texture;
        self.depth_texture_view = depth_texture_view;
    }
    pub fn redraw(&self, world: &World, total_time: f32) {
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
        });

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.texture_bind_group, &[]);
        self.queue.write_buffer(
            &self.time_buffer, 0,
            bytemuck::cast_slice(&[total_time]),
        );
        self.queue.write_buffer(
            &self.view_buffer, 0,
            bytemuck::cast_slice(&self.camera_transform.as_matrix().to_cols_array()),
        );
        render_pass.set_bind_group(1, &self.frame_bind_group, &[]);

        for id in world.meshes.keys().copied().into_iter() {
            let mesh = world.meshes.get(&id).unwrap();
            let transform = world.transforms.get(&id).unwrap();
            let buffered_mesh= self.buffered_meshes.get(&id).unwrap();

            self.queue.write_buffer(&buffered_mesh.transform_buffer, 0, bytemuck::cast_slice(&transform.as_matrix().to_cols_array()));

            render_pass.set_vertex_buffer(0, buffered_mesh.vertex_buffer.slice(..));
            render_pass.set_bind_group(2, &buffered_mesh.bind_group, &[]);
            render_pass.set_index_buffer(buffered_mesh.index_buffer.slice(..), IndexFormat::Uint16);
            render_pass.draw_indexed(0..mesh.indices.len() as u32, 0, 0..1);
        }
        drop(render_pass);

        self.queue.submit(Some(encoder.finish()));
        tex.present();
    }

    pub fn create_buffered_mesh(&mut self, id: u32, mesh: &Mesh) {
        let transform_buffer = self.device.create_buffer_init(&BufferInitDescriptor {
            label: Label::from("transform buffer"),
            contents: &bytemuck::cast_slice(&Mat4::IDENTITY.to_cols_array()),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });
        let bf_mesh = BufferedMesh {
            vertex_buffer: self.device.create_buffer_init(&BufferInitDescriptor {
                label: Label::from("Vertex Buffer"),
                contents: &bytemuck::cast_slice(&mesh.vertexes),
                usage: BufferUsages::VERTEX,
            }),
            index_buffer: self.device.create_buffer_init(&BufferInitDescriptor {
                label: Label::from("Index Buffer"),
                contents: &bytemuck::cast_slice(&mesh.indices),
                usage: BufferUsages::INDEX,
            }),
            bind_group: self.device.create_bind_group(&BindGroupDescriptor {
                label: Label::from("Mesh bindings"),
                layout: &self.mesh_bind_group_layout,
                entries: &[
                    BindGroupEntry {
                        binding: 0,
                        resource: transform_buffer.as_entire_binding(),
                    }
                ],
            }),
            transform_buffer,
        };
        self.buffered_meshes.insert(id, bf_mesh);
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
