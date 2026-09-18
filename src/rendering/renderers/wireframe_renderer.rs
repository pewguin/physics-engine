use std::borrow::Cow;

use wgpu::{BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType, Buffer, BufferBindingType, BufferUsages, ColorTargetState, CompareFunction, DepthStencilState, Device, FragmentState, Label, PipelineLayoutDescriptor, PrimitiveState, RenderPipeline, RenderPipelineDescriptor, ShaderModuleDescriptor, ShaderSource, ShaderStages, TextureFormat, VertexState, util::{BufferInitDescriptor, DeviceExt}};

use crate::rendering::wireframe_vertex::WireframeVertex;

pub struct WireframeRenderer {
    pipeline: RenderPipeline,

    model_matrix_buffer: Buffer,
    positions_buffer: Buffer,
    mesh_bind_group: BindGroup
}

impl WireframeRenderer {
    pub fn new(device: &Device, frame_bind_group_layout: &BindGroupLayout) -> Self {
        let wireframe_shader = device.create_shader_module(ShaderModuleDescriptor { 
            label: Some("wireframe"),
            source: ShaderSource::Wgsl(Cow::Borrowed(include_str!("wireframe.wgsl"))),
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
                // Vertex position array
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::VERTEX,
                    ty: BindingType::Buffer { 
                        ty: BufferBindingType::Storage { 
                            read_only: true 
                        }, 
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let model_matrix_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Label::from("Wireframe model matrix buffer"),
            contents: &[0u8],
            usage: BufferUsages::VERTEX | BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        let positions_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Label::from("Wireframe positions buffer"),
            contents: bytemuck::bytes_of(&0),
            usage: BufferUsages::VERTEX | BufferUsages::STORAGE | BufferUsages::COPY_DST,
        });

        let mesh_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Label::from("Wireframe mesh bind group"),
            layout: &mesh_bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: model_matrix_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: positions_buffer.as_entire_binding(),
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Label::from("Pipeline layout"),
            bind_group_layouts: &[frame_bind_group_layout, &mesh_bind_group_layout,],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Wireframe pipeline"),
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &wireframe_shader,
                entry_point: None,
                compilation_options: Default::default(),
                buffers: &[WireframeVertex::LAYOUT],
            },
            primitive: PrimitiveState {
                cull_mode: None,
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
                module: &wireframe_shader,
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


        Self {
            pipeline,

            model_matrix_buffer,
            positions_buffer,
            mesh_bind_group,
        }
    }

    // pub fn upload_mesh(&self, mesh: &Mesh) -> WireframeMesh {
    //
    // }
}
