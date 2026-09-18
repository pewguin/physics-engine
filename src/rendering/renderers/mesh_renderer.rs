use std::{borrow::Cow, rc::Rc};

use wgpu::{AddressMode, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType, Buffer, BufferBindingType, BufferUsages, ColorTargetState, CompareFunction, DepthStencilState, Device, Extent3d, Face, FilterMode, FragmentState, IndexFormat, Label, Origin3d, PipelineLayoutDescriptor, PrimitiveState, Queue, RenderPass, RenderPipeline, RenderPipelineDescriptor, Sampler, SamplerBindingType, SamplerDescriptor, ShaderModuleDescriptor, ShaderSource, ShaderStages, TexelCopyBufferLayout, TexelCopyTextureInfo, Texture, TextureAspect, TextureDescriptor, TextureDimension, TextureFormat, TextureSampleType, TextureUsages, TextureView, TextureViewDescriptor, TextureViewDimension, VertexState, util::{BufferInitDescriptor, DeviceExt}};

use crate::{physics::mesh::Mesh, rendering::{buffered_mesh::BufferedMesh, material::Material, transform::Transform, vertex::Vertex}};

pub struct MeshRenderer {
    pipeline: RenderPipeline,

    // Bind group layouts
    mesh_bind_group_layout: BindGroupLayout,
    pub material_bind_group_layout: BindGroupLayout,
}

impl MeshRenderer {
    pub fn new(device: &Device, frame_bind_group_layout: &BindGroupLayout) -> Self {
        let shader_module = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("main"),
            source: ShaderSource::Wgsl(Cow::Borrowed(include_str!("shader.wgsl"))),
        });

        let mesh_bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Label::from("Mesh bind group layout"),
            entries: &[
                // Transform
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

        let material_bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor { 
            label: Label::from("Material bind group layout"),
            entries:  &[
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
            bind_group_layouts: &[frame_bind_group_layout, &mesh_bind_group_layout, &material_bind_group_layout],
            push_constant_ranges: &[],
        });

        // Render pipeline for main screen
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
                cull_mode: Some(Face::Back),
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

        Self {
            pipeline,

            mesh_bind_group_layout,
            material_bind_group_layout,
        }
    }

    pub fn draw(&self, frame_bind_group: &BindGroup, render_pass: &mut RenderPass, mesh: &BufferedMesh) {
        render_pass.set_pipeline(&self.pipeline);

        render_pass.set_bind_group(0, frame_bind_group, &[]);
        render_pass.set_bind_group(1, &mesh.mesh_bind_group, &[]);
        render_pass.set_bind_group(2, &mesh.material.bind_group, &[]);

        render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));

        render_pass.set_index_buffer(mesh.index_buffer.slice(..), IndexFormat::Uint16);
        render_pass.draw_indexed(0..mesh.index_count, 0, 0..1);
    }

    pub fn upload_mesh(&self, device: &Device, mesh: &Mesh, transform: &Transform, material: Rc<Material>) -> BufferedMesh {
        let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Label::from("Mesh vertex buffer"),
            contents: bytemuck::cast_slice(&mesh.vertexes),
            usage: BufferUsages::VERTEX | BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        let index_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Label::from("Mesh index buffer"),
            contents: bytemuck::cast_slice(&mesh.indices),
            usage: BufferUsages::INDEX | BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        let transform_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Label::from("Mesh transform buffer"),
            contents: bytemuck::bytes_of(&transform.as_matrix().to_cols_array()),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        let mesh_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Label::from("Mesh bind group"),
            layout: &self.mesh_bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: transform_buffer.as_entire_binding(),
                }
            ],
        });

        BufferedMesh {
            vertex_buffer,
            index_buffer,
            transform_buffer,
            
            index_count: mesh.indices.len() as u32,

            material,
            mesh_bind_group,
        }
    }

    // Load texture from disk to GPU memory
}

