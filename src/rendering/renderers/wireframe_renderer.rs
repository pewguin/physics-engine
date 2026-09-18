use std::borrow::Cow;

use wgpu::{BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType, BlendState, Buffer, BufferBindingType, BufferUsages, ColorTargetState, ColorWrites, CompareFunction, DepthStencilState, Device, FragmentState, Label, PipelineLayoutDescriptor, PrimitiveState, RenderPass, RenderPipeline, RenderPipelineDescriptor, ShaderModuleDescriptor, ShaderSource, ShaderStages, TextureFormat, VertexState, util::{BufferInitDescriptor, DeviceExt}};

use crate::{physics::mesh::Mesh, rendering::{buffered_wireframe_mesh::BufferedWireframeMesh, transform::Transform, vertex::Vertex, wireframe_vertex::WireframeVertex}};

pub struct WireframeRenderer {
    pipeline: RenderPipeline,

    mesh_bind_group_layout: BindGroupLayout,
}

impl WireframeRenderer {
    pub fn new(device: &Device, frame_bind_group_layout: &BindGroupLayout) -> Self {
        let wireframe_shader = device.create_shader_module(ShaderModuleDescriptor { 
            label: Some("wireframe"),
            source: ShaderSource::Wgsl(Cow::Borrowed(include_str!("wireframe.wgsl"))),
        });

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
                buffers: &[],
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
                    blend: Some(BlendState::ALPHA_BLENDING),
                    write_mask: ColorWrites::ALL,
                })],
            }),
            multiview: None,
            cache: None,
        });


        Self {
            pipeline,

            mesh_bind_group_layout,
        }
    }

    pub fn draw(&self, frame_bind_group: &BindGroup, render_pass: &mut RenderPass, mesh: &BufferedWireframeMesh) {
        render_pass.set_pipeline(&self.pipeline);

        render_pass.set_bind_group(0, frame_bind_group, &[]);
        render_pass.set_bind_group(1, &mesh.mesh_bind_group, &[]);

        render_pass.draw(0..mesh.position_count,  0..1);
    }

    pub fn upload_mesh(&self, device: &Device, mesh: &Mesh, transform: &Transform) -> BufferedWireframeMesh {
        let transform_buffer = device.create_buffer_init(&BufferInitDescriptor {
                label: Label::from("Wireframe transform buffer"),
                contents: bytemuck::bytes_of(&transform.as_matrix().to_cols_array()),
                usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        let verts: Vec<WireframeVertex> = mesh.indices.iter()
            .map(|idx| {
                let vert = mesh.vertexes[*idx as usize];
                WireframeVertex { position: vert.position }
            })
            .collect();

        let positions_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Label::from("Wireframe positions buffer"),
            contents: bytemuck::cast_slice(&verts),
            usage: BufferUsages::VERTEX | BufferUsages::STORAGE | BufferUsages::COPY_DST,
        });

        let mesh_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Label::from("Wireframe mesh bind group"),
            layout: &self.mesh_bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: transform_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: positions_buffer.as_entire_binding(),
                },
            ],
        });
        BufferedWireframeMesh {
            positions_buffer,
            transform_buffer,
            position_count: verts.len() as u32,

            mesh_bind_group,
        }
    }
}
