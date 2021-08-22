use wgpu::MultisampleState;

use crate::{
    buffer::{IndexedVertexBuffer, StagingFactory, VertexTypedBuffer},
    vertex::TexturedVertex,
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

pub struct ClownColorsScene {
    pipeline: wgpu::RenderPipeline,
    vertex_buffer: IndexedVertexBuffer<TexturedVertex>,
}

impl Scene for ClownColorsScene {
    fn new(
        device: &wgpu::Device,
        _queue: &wgpu::Queue,
        sc: &wgpu::SurfaceConfiguration,
        _staging: &mut StagingFactory,
    ) -> Self {
        let vert2_module = device
            .create_shader_module(&wgpu::include_spirv!("../shaders/mysecondshader.vert.spv"));
        let frag2_module = device
            .create_shader_module(&wgpu::include_spirv!("../shaders/mysecondshader.frag.spv"));

        let vertex_buffer = IndexedVertexBuffer::from_vertices_indexes(
            device,
            VERTICES_1,
            INDICES_1,
            Some("San Cheese Is Running Over Your Vertices"),
            Some("San Cheese Is Indexing You"),
        );

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout #1"),
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });

        /*let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline #2"),
            layout: Some(&pipeline_layout),
            vertex_stage: wgpu::ProgrammableStageDescriptor {
                module: &vert2_module,
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
                module: &frag2_module,
                entry_point: "main",
            }),
            primitive_topology: wgpu::PrimitiveTopology::TriangleList,
            color_states: &[wgpu::ColorStateDescriptor {
                format: sc.format,
                alpha_blend: wgpu::BlendDescriptor::REPLACE,
                color_blend: wgpu::BlendDescriptor::REPLACE,
                write_mask: wgpu::ColorWrite::ALL,
            }],
            depth_stencil_state: None,
            vertex_state: wgpu::VertexStateDescriptor {
                index_format: wgpu::IndexFormat::Uint16,
                vertex_buffers: &[vertex_buffer.descriptor()],
            },
            sample_count: 1,
            sample_mask: !0,
            alpha_to_coverage_enabled: false,
        });*/

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Clown - Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &vert2_module,
                entry_point: "main",
                buffers: &[vertex_buffer.descriptor()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &frag2_module,
                entry_point: "main",
                targets: &[wgpu::ColorTargetState {
                    format: sc.format,
                    blend: Some(wgpu::BlendState::REPLACE),
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
            multisample: MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
        });

        Self {
            pipeline,
            vertex_buffer,
        }
    }

    fn input(&mut self, _event: &winit::event::WindowEvent) -> bool {
        false
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
            color_attachments: &[wgpu::RenderPassColorAttachment {
                view: frame_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(state.bg_color),
                    store: true,
                },
            }],
            depth_stencil_attachment: None,
            label: Some("Clown - Main render pass"),
        };

        let mut render_pass = encoder.begin_render_pass(rp_desc);
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.vertices.slice(..));
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
