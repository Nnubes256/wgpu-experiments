use crate::{
    buffer::{StagingFactory, VertexBuffer, VertexTypedBuffer},
    vertex::FlatVertex,
    GlobalState,
};

use super::Scene;

const VERTICES_3: &[FlatVertex] = &[
    FlatVertex {
        position: [0.3, 0.3, 0.0],
        color: [1.0, 0.0, 0.0],
    },
    FlatVertex {
        position: [-0.5, 0.0, 0.0],
        color: [0.0, 1.0, 0.0],
    },
    FlatVertex {
        position: [-0.25, -0.25, 0.0],
        color: [1.0, 0.0, 0.0],
    },
    FlatVertex {
        position: [-0.25, -0.25, 0.0],
        color: [0.0, 0.0, 1.0],
    },
    FlatVertex {
        position: [0.0, -0.6, 0.0],
        color: [0.0, 1.0, 0.0],
    },
    FlatVertex {
        position: [0.3, 0.3, 0.0],
        color: [0.0, 0.0, 1.0],
    },
];

pub struct TriangleScene {
    pipeline: wgpu::RenderPipeline,
    vertex_buffer: VertexBuffer<FlatVertex>,
}

impl Scene for TriangleScene {
    fn new(
        device: &wgpu::Device,
        _queue: &wgpu::Queue,
        sc: &wgpu::SurfaceConfiguration,
        _staging: &mut StagingFactory,
    ) -> Self {
        let vert3_module =
            device.create_shader_module(&wgpu::include_spirv!("../shaders/dima.vert.spv"));
        let frag3_module =
            device.create_shader_module(&wgpu::include_spirv!("../shaders/dima.frag.spv"));

        let vertex_buffer =
            VertexBuffer::from_vertices(device, VERTICES_3, Some("Funny Triangle - Vertex Buffer"));

        let pipeline_layout1 = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Funny Triangle - Pipeline Layout"),
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });

        /*let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Funny Triangle - Render Pipeline"),
            layout: Some(&pipeline_layout1),
            vertex_stage: wgpu::ProgrammableStageDescriptor {
                module: &vert3_module,
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
                module: &frag3_module,
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
            label: Some("San Cheese Is Laying Your Pipes"),
            layout: Some(&pipeline_layout1),
            vertex: wgpu::VertexState {
                module: &vert3_module,
                entry_point: "main",
                buffers: &[vertex_buffer.descriptor()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &frag3_module,
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
            multisample: wgpu::MultisampleState {
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
        state: &GlobalState,
        _staging: &StagingFactory,
    ) -> Result<(), wgpu::SurfaceError> {
        let rp_desc = &wgpu::RenderPassDescriptor {
            label: Some("Funny Triangle - Render Pass Descriptor"),
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
        render_pass.set_vertex_buffer(0, self.vertex_buffer.buffer.slice(..));
        render_pass.draw(0..self.vertex_buffer.len, 0..1);

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
