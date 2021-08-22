use winit::event::{ElementState, KeyboardInput, VirtualKeyCode, WindowEvent};

use crate::{
    buffer::{IndexedVertexBuffer, StagingFactory},
    texture::Texture,
    vertex::{Descriptable, TexturedVertex},
};

use super::Scene;

const VERTICES_1: &[TexturedVertex] = &[
    TexturedVertex {
        position: [0.0, 0.0, 0.0],
        tex_coords: [0.5, 0.5],
    },
    TexturedVertex {
        position: [-0.5, 0.0, 0.0],
        tex_coords: [0.0, 0.5],
    },
    TexturedVertex {
        position: [-0.25, -0.5, 0.0],
        tex_coords: [0.25, 1.0],
    },
    TexturedVertex {
        position: [0.25, -0.5, 0.0],
        tex_coords: [0.75, 1.0],
    },
    TexturedVertex {
        position: [0.5, 0.0, 0.0],
        tex_coords: [1.0, 0.5],
    },
    TexturedVertex {
        position: [0.25, 0.5, 0.0],
        tex_coords: [0.75, 0.0],
    },
    TexturedVertex {
        position: [-0.25, 0.5, 0.0],
        tex_coords: [0.25, 0.0],
    },
];

const INDICES_1: &[u16] = &[0, 1, 2, 0, 2, 3, 0, 3, 4, 0, 4, 5, 0, 5, 6, 0, 6, 1];

#[derive(Debug)]
enum SelectedImage {
    SanCheese,
    Nnubes,
}

pub struct TextureExampleScene {
    pipeline: wgpu::RenderPipeline,
    vertex_buffer: IndexedVertexBuffer<TexturedVertex>,
    diffuse1_bind_group: wgpu::BindGroup,
    _diffuse1_texture: Texture,
    diffuse2_bind_group: wgpu::BindGroup,
    _diffuse2_texture: Texture,
    selected_image: SelectedImage,
}

impl Scene for TextureExampleScene {
    fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        sc: &wgpu::SurfaceConfiguration,
        _staging: &mut StagingFactory,
    ) -> Self {
        let diffuse1_bytes = include_bytes!("../../assets/sanCheese.png");
        let diffuse1_texture =
            Texture::from_bytes(device, queue, diffuse1_bytes, "San Cheese Is Watching You")
                .unwrap();

        let diffuse2_bytes = include_bytes!("../../assets/nnubes256.png");
        let diffuse2_texture =
            Texture::from_bytes(device, queue, diffuse2_bytes, "Nnubes256 Is Watching You")
                .unwrap();

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("San Cheese Is Laying Your Bounds"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: false },
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler {
                            filtering: true,
                            comparison: false,
                        },
                        count: None,
                    },
                ],
            });

        let diffuse1_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("San Cheese Is Binding You"),
            layout: &texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&diffuse1_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&diffuse1_texture.sampler),
                },
            ],
        });

        let diffuse2_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("San Cheese Is Binding You"),
            layout: &texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&diffuse2_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&diffuse2_texture.sampler),
                },
            ],
        });

        let vertex_buffer = IndexedVertexBuffer::from_vertices_indexes(
            device,
            VERTICES_1,
            INDICES_1,
            Some("San Cheese Is Running Over Your Vertices"),
            Some("San Cheese Is Indexing You"),
        );

        let vert1_module =
            device.create_shader_module(&wgpu::include_spirv!("../shaders/myfirstshader.vert.spv"));
        let frag1_module =
            device.create_shader_module(&wgpu::include_spirv!("../shaders/myfirstshader.frag.spv"));

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("San Cheese Is Planning Your Pipes"),
            bind_group_layouts: &[&texture_bind_group_layout],
            push_constant_ranges: &[],
        });

        /*let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("San Cheese Is Laying Your Pipes"),
            layout: Some(&pipeline_layout),
            vertex_stage: wgpu::ProgrammableStageDescriptor {
                module: &vert1_module,
                entry_point: "main",
            },
            rasterization_state: Some(wgpu::RasterizationStateDescriptor {
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: wgpu::CullMode::Back,
                depth_bias: 0,
                depth_bias_slope_scale: 0.0,
                depth_bias_clamp: 0.0,
                clamp_depth: false,
            }),
            fragment_stage: Some(wgpu::ProgrammableStageDescriptor {
                module: &frag1_module,
                entry_point: "main",
            }),
            primitive_topology: wgpu::PrimitiveTopology::TriangleList,
            color_states: &[wgpu::ColorStateDescriptor {
                format: sc.format,
                color_blend: wgpu::BlendDescriptor {
                    src_factor: wgpu::BlendFactor::SrcAlpha,
                    dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                    operation: wgpu::BlendOperation::Add,
                },
                alpha_blend: wgpu::BlendDescriptor {
                    src_factor: wgpu::BlendFactor::One,
                    dst_factor: wgpu::BlendFactor::Zero,
                    operation: wgpu::BlendOperation::Add,
                },
                write_mask: wgpu::ColorWrite::ALL,
            }],
            depth_stencil_state: None,
            vertex_state: wgpu::VertexStateDescriptor {
                index_format: wgpu::IndexFormat::Uint16,
                vertex_buffers: &[TexturedVertex::descriptor()],
            },
            sample_count: 1,
            sample_mask: !0,
            alpha_to_coverage_enabled: false,
        });*/

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("San Cheese Is Laying Your Pipes"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &vert1_module,
                entry_point: "main",
                buffers: &[TexturedVertex::descriptor()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &frag1_module,
                entry_point: "main",
                targets: &[wgpu::ColorTargetState {
                    format: sc.format,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent::OVER,
                        alpha: wgpu::BlendComponent::REPLACE,
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                }],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                clamp_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
        });

        Self {
            pipeline,
            vertex_buffer,
            diffuse1_bind_group,
            _diffuse1_texture: diffuse1_texture,
            diffuse2_bind_group,
            _diffuse2_texture: diffuse2_texture,
            selected_image: SelectedImage::Nnubes,
        }
    }

    fn input(&mut self, event: &winit::event::WindowEvent) -> bool {
        match event {
            WindowEvent::KeyboardInput { input, .. } => {
                if let KeyboardInput {
                    state: ElementState::Pressed,
                    virtual_keycode: Some(VirtualKeyCode::N),
                    ..
                } = input
                {
                    println!("Pressed N");
                    self.selected_image = match self.selected_image {
                        SelectedImage::SanCheese => SelectedImage::Nnubes,
                        SelectedImage::Nnubes => SelectedImage::SanCheese,
                    };
                    println!("{:?}", self.selected_image);
                }

                true
            }
            _ => false,
        }
    }

    fn update(&mut self, _device: &wgpu::Device, _queue: &wgpu::Queue) {}

    fn render(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        frame_view: &wgpu::TextureView,
        state: &crate::GlobalState,
        _staging: &StagingFactory,
    ) -> Result<(), wgpu::SurfaceError> {
        let rp_desc = &wgpu::RenderPassDescriptor {
            label: Some("Textured - Render Pass Descriptor"),
            color_attachments: &[wgpu::RenderPassColorAttachment {
                view: frame_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(state.bg_color),
                    store: true,
                },
            }],
            depth_stencil_attachment: None,
        };

        let mut render_pass = encoder.begin_render_pass(rp_desc);
        render_pass.set_pipeline(&self.pipeline);

        render_pass.set_vertex_buffer(0, self.vertex_buffer.vertices.slice(..));

        let selected_bind_group = match self.selected_image {
            SelectedImage::SanCheese => &self.diffuse1_bind_group,
            SelectedImage::Nnubes => &self.diffuse2_bind_group,
        };

        render_pass.set_bind_group(0, selected_bind_group, &[]);
        render_pass.set_index_buffer(
            self.vertex_buffer.indices.slice(..),
            wgpu::IndexFormat::Uint16,
        );
        render_pass.draw_indexed(0..self.vertex_buffer.num_indices, 0, 0..1);

        Ok(())
    }

    fn resize(
        &mut self,
        _device: &wgpu::Device,
        _queue: &wgpu::Queue,
        _size: winit::dpi::PhysicalSize<u32>,
    ) {
    }
}
