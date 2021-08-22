use cgmath::{Deg, Euler, Quaternion};
use winit::event::{ElementState, KeyboardInput, VirtualKeyCode, WindowEvent};

use crate::{
    buffer::{IndexedVertexBuffer, OldUniform, StagingFactory},
    camera::{Camera, CameraController, CameraUniform},
    mesh::{OldMesh, Transform},
    texture::Texture,
    transform,
    vertex::{Descriptable, TexturedVertex},
};

use super::Scene;

const VERTICES_1: &[TexturedVertex] = &[
    // 0
    TexturedVertex {
        position: [0.0, 0.0, 0.5],
        tex_coords: [0.5, 0.5],
    },
    TexturedVertex {
        position: [-0.5, 0.0, 0.5],
        tex_coords: [0.0, 0.5],
    },
    TexturedVertex {
        position: [-0.25, -0.5, 0.5],
        tex_coords: [0.25, 1.0],
    },
    TexturedVertex {
        position: [0.25, -0.5, 0.5],
        tex_coords: [0.75, 1.0],
    },
    TexturedVertex {
        position: [0.5, 0.0, 0.5],
        tex_coords: [1.0, 0.5],
    },
    TexturedVertex {
        position: [0.25, 0.5, 0.5],
        tex_coords: [0.75, 0.0],
    },
    TexturedVertex {
        position: [-0.25, 0.5, 0.5],
        tex_coords: [0.25, 0.0],
    },
    // 7
    TexturedVertex {
        position: [0.0, 0.0, -0.5],
        tex_coords: [0.5, 0.5],
    },
    TexturedVertex {
        position: [-0.5, 0.0, -0.5],
        tex_coords: [1.0, 0.5],
    },
    TexturedVertex {
        position: [-0.25, -0.5, -0.5],
        tex_coords: [0.75, 1.0],
    },
    TexturedVertex {
        position: [0.25, -0.5, -0.5],
        tex_coords: [0.25, 1.0],
    },
    TexturedVertex {
        position: [0.5, 0.0, -0.5],
        tex_coords: [0.0, 0.5],
    },
    TexturedVertex {
        position: [0.25, 0.5, -0.5],
        tex_coords: [0.25, 0.0],
    },
    TexturedVertex {
        position: [-0.25, 0.5, -0.5],
        tex_coords: [0.75, 0.0],
    },
];

#[rustfmt::skip]
const INDICES_1: &[u16] = &[
    0, 1, 2,
    0, 2, 3,
    0, 3, 4,
    0, 4, 5,
    0, 5, 6,
    0, 6, 1,
    7, 9, 8,
    7, 10, 9,
    7, 11, 10,
    7, 12, 11,
    7, 13, 12,
    7, 8, 13,
    1, 8, 9,
    1, 8, 2,
    2, 9, 10,
    2, 9, 3,
    3, 10, 11,
    3, 10, 4,
    4, 11, 12,
    4, 11, 5,
    5, 12, 13,
    5, 12, 6,
    6, 13, 8,
    6, 13, 1
];

const UNIFORM_MATRIX_BELT: &str = "camera.belt";

#[derive(Debug)]
enum SelectedImage {
    SanCheese,
    Nnubes,
}

pub struct CameraScene {
    pipeline: wgpu::RenderPipeline,
    epic_mesh: OldMesh<TexturedVertex>,
    diffuse1_bind_group: wgpu::BindGroup,
    _diffuse1_texture: Texture,
    diffuse2_bind_group: wgpu::BindGroup,
    _diffuse2_texture: Texture,
    selected_image: SelectedImage,
    camera: Camera,
    camera_controller: CameraController,
    camera_uniform: CameraUniform,
    camera_uniform_buffer: wgpu::Buffer,
    epic_mesh_uniform_buffer: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,
}

impl Scene for CameraScene {
    fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        sc: &wgpu::SurfaceConfiguration,
        staging: &mut StagingFactory,
    ) -> Self {
        let diffuse1_bytes = include_bytes!("../../assets/sanCheese.png");
        let diffuse1_texture =
            Texture::from_bytes(device, queue, diffuse1_bytes, "San Cheese Is Watching You")
                .unwrap();

        let diffuse2_bytes = include_bytes!("../../assets/nnubes256.png");
        let diffuse2_texture =
            Texture::from_bytes(device, queue, diffuse2_bytes, "Nnubes256 Is Watching You")
                .unwrap();

        staging.create_stager(UNIFORM_MATRIX_BELT.to_owned(), 64);

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
            label: Some("Nnubes256 Is Binding You"),
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

        let mesh_transform = transform! {
            t: [0.0, 0.0, 0.0],
            r: [0.0, 0.0, 0.0],
            s: [1.0, 1.0, 1.0]
        };

        let epic_mesh = OldMesh::new(vertex_buffer, mesh_transform);

        let vert1_module =
            device.create_shader_module(&wgpu::include_spirv!("../shaders/camerabois.vert.spv"));
        let frag1_module =
            device.create_shader_module(&wgpu::include_spirv!("../shaders/camerabois.frag.spv"));

        let camera = Camera {
            eye: (0.0, 1.0, 2.0).into(),
            target: (0.0, 0.0, 0.0).into(),
            up: cgmath::Vector3::unit_y(),
            aspect: sc.width as f32 / sc.height as f32,
            fovy: 45.0,
            znear: 0.1,
            zfar: 100.0,
        };

        let camera_controller = CameraController::new(0.2);

        let mut camera_uniform = CameraUniform::default();
        camera_uniform.update(&camera);

        let mesh_uniform_buf = epic_mesh.transform().as_buffer(
            device,
            Some("Cameras - Epic Model Transform Uniform Buffer"),
        );

        let camera_uniform_buf =
            camera_uniform.into_buffer(device, Some("Cameras - Camera Uniform Buffer"));

        let uniform_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Cameras - Camera Uniform Bind Group Layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            });

        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Cameras - Camera Uniform Bind Group Layout"),
            layout: &uniform_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &camera_uniform_buf,
                        offset: 0,
                        size: None,
                    }),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &mesh_uniform_buf,
                        offset: 0,
                        size: None,
                    }),
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("San Cheese Is Planning Your Pipes"),
            bind_group_layouts: &[&texture_bind_group_layout, &uniform_bind_group_layout],
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
            epic_mesh,
            diffuse1_bind_group,
            _diffuse1_texture: diffuse1_texture,
            diffuse2_bind_group,
            _diffuse2_texture: diffuse2_texture,
            selected_image: SelectedImage::Nnubes,
            camera,
            camera_controller,
            camera_uniform,
            camera_uniform_buffer: camera_uniform_buf,
            epic_mesh_uniform_buffer: mesh_uniform_buf,
            uniform_bind_group,
        }
    }

    fn input(&mut self, event: &winit::event::WindowEvent) -> bool {
        let camera_handled = self.camera_controller.input(event);
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
            _ => camera_handled,
        }
    }

    fn update(&mut self, _device: &wgpu::Device, _queue: &wgpu::Queue) {
        // Update the camera based on the input state
        self.camera_controller.update(&mut self.camera);

        // Update the projection buffer based on the camera's updated state
        self.camera_uniform.update(&self.camera);

        self.epic_mesh.transform_mut().set_rotation(|r| {
            *r = (*r)
                * Quaternion::from(Euler {
                    x: Deg(0.0),
                    y: Deg(1.0),
                    z: Deg(0.0),
                });
        });
    }

    fn render(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        frame_view: &wgpu::TextureView,
        state: &crate::GlobalState,
        staging: &StagingFactory,
    ) -> Result<(), wgpu::SurfaceError> {
        let rp_desc = &wgpu::RenderPassDescriptor {
            label: Some("Camera Demo - Render Pass"),
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

        let mut stager = staging.fetch_stager(UNIFORM_MATRIX_BELT);
        stager.write_buffer(
            encoder,
            &self.camera_uniform_buffer,
            0,
            bytemuck::bytes_of(&self.camera_uniform),
        );
        stager.write_buffer(
            encoder,
            &self.epic_mesh_uniform_buffer,
            0,
            bytemuck::bytes_of(&self.epic_mesh.transform().uniform_matrix()),
        );

        let mut render_pass = encoder.begin_render_pass(rp_desc);
        render_pass.set_pipeline(&self.pipeline);

        let selected_bind_group = match self.selected_image {
            SelectedImage::SanCheese => &self.diffuse1_bind_group,
            SelectedImage::Nnubes => &self.diffuse2_bind_group,
        };
        render_pass.set_bind_group(0, selected_bind_group, &[]);
        render_pass.set_bind_group(1, &self.uniform_bind_group, &[]);

        self.epic_mesh.render(&mut render_pass, 0..1);

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
