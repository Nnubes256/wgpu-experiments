use std::num::NonZeroU64;

use cgmath::MetricSpace;
use wgpu::BufferBinding;
use winit::event::{ElementState, KeyboardInput, VirtualKeyCode, WindowEvent};

use crate::{
    buffer::{IndexedVertexBuffer, InstanceVertexBuffer, OldUniform, StagingFactory},
    camera::{Camera, CameraController, CameraUniform},
    mesh::Transform,
    texture::{DepthTexture, Texture},
    transform,
    vertex::{Descriptable, TexturedVertex, VertexBufferable},
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
    1, 9, 2,
    2, 9, 10,
    2, 10, 3,
    3, 10, 11,
    3, 11, 4,
    4, 11, 12,
    4, 12, 5,
    5, 12, 13,
    5, 13, 6,
    6, 13, 8,
    6, 8, 1
];

const CAMERA_BELT: &str = "instancing.camera";
const INSTANCE_BELT: &str = "instancing.instances";

#[derive(Debug)]
pub struct Instance {
    transform: Transform,
}

#[repr(C, packed)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct InstanceVertex {
    transform: [[f32; 4]; 4],
}

impl From<&Instance> for InstanceVertex {
    fn from(i: &Instance) -> Self {
        InstanceVertex {
            transform: i.transform.uniform_matrix(),
        }
    }
}

impl VertexBufferable for InstanceVertex {}

impl Descriptable for InstanceVertex {
    fn descriptor<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<InstanceVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    // While our vertex shader only uses locations 0, and 1 now, in later tutorials we'll
                    // be using 2, 3, and 4, for Vertex. We'll start at slot 5 not conflict with them later
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float32x4,
                },
                // A mat4 takes up 4 vertex slots as it is technically 4 vec4s. We need to define a slot
                // for each vec4. We don't have to do this in code though.
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 6,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 7,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 12]>() as wgpu::BufferAddress,
                    shader_location: 8,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}

pub struct Mesh {
    data: IndexedVertexBuffer<TexturedVertex>,
}

impl Mesh {
    pub fn new(data: IndexedVertexBuffer<TexturedVertex>) -> Self {
        Self { data }
    }

    pub fn render<'a>(
        &'a self,
        render_pass: &mut wgpu::RenderPass<'a>,
        instances: Option<&'a InstanceVertexBuffer<InstanceVertex>>,
    ) {
        render_pass.set_vertex_buffer(0, self.data.vertices.slice(..));
        render_pass.set_index_buffer(self.data.indices.slice(..), wgpu::IndexFormat::Uint16);
        if let Some(instances) = instances {
            render_pass.set_vertex_buffer(1, instances.buffer.slice(..));
            render_pass.draw_indexed(0..self.data.num_indices, 0, 0..instances.len)
        } else {
            render_pass.draw_indexed(0..self.data.num_indices, 0, 0..1)
        }
    }
}

#[derive(Copy, Clone, Debug)]
enum SelectedImage {
    SanCheese,
    Nnubes,
}

#[derive(Copy, Clone, Debug)]
enum SelectedAnimation {
    DoubleWave,
    Metaball,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum SelectedExtraPass {
    None,
    Depth,
}

struct DepthPass {
    pipeline: wgpu::RenderPipeline,
    texture: DepthTexture,
    bind_group: wgpu::BindGroup,
    bind_group_layout: wgpu::BindGroupLayout,
}

impl DepthPass {
    fn new(
        device: &wgpu::Device,
        _queue: &wgpu::Queue,
        sc: &wgpu::SurfaceConfiguration,
        _staging: &mut StagingFactory,
    ) -> Self {
        let vert1_module = device.create_shader_module(&wgpu::include_spirv!(
            "../shaders/instancing_depth.vert.spv"
        ));
        let frag1_module = device.create_shader_module(&wgpu::include_spirv!(
            "../shaders/instancing_depth.frag.spv"
        ));

        let texture = DepthTexture::from_screen(
            device,
            sc.width,
            sc.height,
            Some("Instancing - Depth Texture"),
        );

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Instancing - Depth Pass - Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Depth,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler {
                        filtering: true,
                        comparison: true,
                    },
                    count: None,
                },
            ],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Instancing - Depth Pass - Bind Group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&texture.sampler),
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Instancing - Depth Pass - Render Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        /*let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Instancing - Depth Pass - Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex_stage: wgpu::ProgrammableStageDescriptor {
                module: &vert1_module,
                entry_point: "main",
            },
            fragment_stage: Some(wgpu::ProgrammableStageDescriptor {
                module: &frag1_module,
                entry_point: "main",
            }),
            rasterization_state: Some(wgpu::RasterizationStateDescriptor {
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: wgpu::CullMode::Back,
                depth_bias: 0,
                depth_bias_slope_scale: 0.0,
                depth_bias_clamp: 0.0,
                clamp_depth: false,
            }),
            primitive_topology: wgpu::PrimitiveTopology::TriangleList,
            color_states: &[wgpu::ColorStateDescriptor {
                format: sc.format,
                color_blend: wgpu::BlendDescriptor::REPLACE,
                alpha_blend: wgpu::BlendDescriptor::REPLACE,
                write_mask: wgpu::ColorWrite::ALL,
            }],
            depth_stencil_state: None,
            vertex_state: wgpu::VertexStateDescriptor {
                index_format: wgpu::IndexFormat::Uint16,
                vertex_buffers: &[],
            },
            sample_count: 1,
            sample_mask: !0,
            alpha_to_coverage_enabled: false,
        });*/

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Instancing - Depth Pass - Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &vert1_module,
                entry_point: "main",
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &frag1_module,
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
            texture,
            bind_group_layout,
            bind_group,
        }
    }

    pub fn resize(&mut self, device: &wgpu::Device, size: winit::dpi::PhysicalSize<u32>) {
        self.texture = DepthTexture::from_screen(
            device,
            size.width,
            size.height,
            Some("Instancing - Depth Texture"),
        );

        self.bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Instancing - Depth Pass - Bind Group"),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&self.texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.texture.sampler),
                },
            ],
        });
    }

    pub fn render(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        frame_view: &wgpu::TextureView,
        _state: &crate::GlobalState,
    ) -> Result<(), wgpu::SurfaceError> {
        let rp_desc = &wgpu::RenderPassDescriptor {
            label: Some("Depth pass"),
            color_attachments: &[wgpu::RenderPassColorAttachment {
                view: frame_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: true,
                },
            }],
            depth_stencil_attachment: None,
        };

        let mut render_pass = encoder.begin_render_pass(rp_desc);

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.bind_group, &[]);
        render_pass.draw(0..6, 0..1);

        Ok(())
    }
}

pub struct InstancesScene {
    pipeline: wgpu::RenderPipeline,
    instances: Vec<Instance>,
    instances_buffer: InstanceVertexBuffer<InstanceVertex>,
    epic_mesh: Mesh,
    diffuse1_bind_group: wgpu::BindGroup,
    _diffuse1_texture: Texture,
    diffuse2_bind_group: wgpu::BindGroup,
    _diffuse2_texture: Texture,
    depth_pass: DepthPass,
    selected_image: SelectedImage,
    selected_animation: SelectedAnimation,
    selected_pass: SelectedExtraPass,
    camera: Camera,
    camera_controller: CameraController,
    camera_uniform: CameraUniform,
    camera_uniform_buffer: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,
    time: f64,
}

impl Scene for InstancesScene {
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

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("San Cheese Is Laying Your Bounds"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
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

        let epic_mesh = Mesh::new(vertex_buffer);

        let vert1_module =
            device.create_shader_module(&wgpu::include_spirv!("../shaders/instancing.vert.spv"));
        let frag1_module =
            device.create_shader_module(&wgpu::include_spirv!("../shaders/instancing.frag.spv"));

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
        staging.create_stager(CAMERA_BELT.to_owned(), 64);

        let mut instances = Vec::with_capacity(128);
        staging.create_stager(INSTANCE_BELT.to_owned(), 128 * 64);

        for i in -16..=16 {
            for j in -16..=16 {
                let x = i as f32;
                let y = j as f32;
                instances.push(Instance {
                    transform: transform!(
                        t: [x, y, 0.0],
                        r: [0.0, 0.0, 0.0],
                        s: [1.0, 1.0, 1.0]
                    ),
                });
            }
        }

        //println!("{:?}", instances);

        let instances_buffer = InstanceVertexBuffer::from_instances(
            device,
            &instances,
            Some("Instances - Instances Vertex Buffer"),
        );

        let camera_uniform_buf =
            camera_uniform.into_buffer(device, Some("Cameras - Camera Uniform Buffer"));

        let uniform_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Cameras - Camera Uniform Bind Group Layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Cameras - Camera Uniform Bind Group Layout"),
            layout: &uniform_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(BufferBinding {
                    buffer: &camera_uniform_buf,
                    offset: 0,
                    size: None,
                }),
            }],
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
            depth_stencil_state: Some(wgpu::DepthStencilStateDescriptor {
                format: DepthTexture::DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilStateDescriptor::default(),
            }),
            vertex_state: wgpu::VertexStateDescriptor {
                index_format: wgpu::IndexFormat::Uint16,
                vertex_buffers: &[TexturedVertex::descriptor(), InstanceVertex::descriptor()],
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
                buffers: &[TexturedVertex::descriptor(), InstanceVertex::descriptor()],
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
            depth_stencil: Some(wgpu::DepthStencilState {
                format: DepthTexture::DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
        });

        let depth_pass = DepthPass::new(device, queue, sc, staging);

        Self {
            pipeline,
            epic_mesh,
            instances,
            instances_buffer,
            diffuse1_bind_group,
            _diffuse1_texture: diffuse1_texture,
            diffuse2_bind_group,
            _diffuse2_texture: diffuse2_texture,
            depth_pass,
            selected_image: SelectedImage::Nnubes,
            selected_animation: SelectedAnimation::DoubleWave,
            selected_pass: SelectedExtraPass::None,
            camera,
            camera_controller,
            camera_uniform,
            camera_uniform_buffer: camera_uniform_buf,
            uniform_bind_group,
            time: 0.0,
        }
    }

    fn input(&mut self, event: &winit::event::WindowEvent) -> bool {
        let camera_handled = self.camera_controller.input(event);
        match event {
            WindowEvent::KeyboardInput { input, .. } => {
                if let KeyboardInput {
                    state: ElementState::Pressed,
                    virtual_keycode: Some(keycode),
                    ..
                } = input
                {
                    match keycode {
                        VirtualKeyCode::N => {
                            println!("Pressed N");
                            self.selected_image = match self.selected_image {
                                SelectedImage::SanCheese => SelectedImage::Nnubes,
                                SelectedImage::Nnubes => SelectedImage::SanCheese,
                            };
                            println!("{:?}", self.selected_image);

                            true
                        }
                        VirtualKeyCode::M => {
                            println!("Pressed M");
                            self.selected_animation = match self.selected_animation {
                                SelectedAnimation::DoubleWave => SelectedAnimation::Metaball,
                                SelectedAnimation::Metaball => SelectedAnimation::DoubleWave,
                            };
                            println!("{:?}", self.selected_animation);

                            true
                        }
                        VirtualKeyCode::B => {
                            println!("Pressed B");
                            self.selected_pass = match self.selected_pass {
                                SelectedExtraPass::None => SelectedExtraPass::Depth,
                                SelectedExtraPass::Depth => SelectedExtraPass::None,
                            };
                            println!("{:?}", self.selected_pass);

                            true
                        }
                        _ => false,
                    }
                } else {
                    camera_handled
                }
            }
            _ => camera_handled,
        }
    }

    fn update(&mut self, _device: &wgpu::Device, _queue: &wgpu::Queue) {
        // Update the camera based on the input state
        self.camera_controller.update(&mut self.camera);

        // Update the projection buffer based on the camera's updated state
        self.camera_uniform.update(&self.camera);

        // Write directly to the camera's uniform buffer

        // This makes Xcode cry

        for (i, instance) in self.instances.iter_mut().enumerate() {
            let i_x = i % 33;
            let i_y = (i + 1) / 33;
            #[inline]
            fn metaballs(x: usize, y: usize, t: f64) -> f32 {
                const MIN_DIST: f32 = 1e-3;
                const RADIUS: f32 = 8.0;

                let cx = ((t / 120.0) + std::f64::consts::PI / 2.0).sin() as f32 * 15.0 + 16.0;
                let cy = (t / 120.0).sin() as f32 * 15.0 + 16.0;

                let i_vector = cgmath::Vector2::new(x as f32, y as f32);
                let center = cgmath::Vector2::new(cx, cy);
                let distance = i_vector.distance(center);
                ((2.0 * RADIUS) / distance.max(MIN_DIST)).min(8.0)
            }
            #[inline]
            fn double_wave(x: usize, y: usize, t: f64) -> f32 {
                ((t / 120.0) + (((x + y + 2) as f64) / 4.0)).sin() as f32
            }

            let sel = self.selected_animation;
            let time = self.time;

            instance.transform.set_translation(|t| match sel {
                SelectedAnimation::DoubleWave => {
                    t.z = double_wave(i_x, i_y, time);
                }
                SelectedAnimation::Metaball => {
                    t.z = metaballs(i_x, i_y, time);
                }
            });

            /*self.instances_buffer
            .copy_instance(queue, instance, i as wgpu::BufferAddress);*/
        }

        self.time += 1.0;
    }

    //fn recall(&mut self) {}

    fn render(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        frame_view: &wgpu::TextureView,
        state: &crate::GlobalState,
        staging: &StagingFactory,
    ) -> Result<(), wgpu::SurfaceError> {
        let mut camera_stager = staging.fetch_stager(CAMERA_BELT);
        camera_stager.write_buffer(
            encoder,
            &self.camera_uniform_buffer,
            0,
            bytemuck::bytes_of(&self.camera_uniform),
        );

        let potential_size = NonZeroU64::new(
            self.instances.len() as wgpu::BufferAddress
                * self.instances_buffer.descriptor().array_stride,
        );
        if let Some(size) = potential_size {
            let mut instance_stager = staging.fetch_stager(INSTANCE_BELT);
            let mut staging_buffer = instance_stager.create_staging_area(
                encoder,
                &self.instances_buffer.buffer,
                0,
                size,
            );
            for (i, instance) in self.instances.iter().enumerate() {
                self.instances_buffer
                    .copy_instance_into_view(&mut staging_buffer, instance, i);
            }
        }

        {
            let rp_desc = &wgpu::RenderPassDescriptor {
                label: Some("Instancing - Render Pass Descriptor"),
                color_attachments: &[wgpu::RenderPassColorAttachment {
                    view: frame_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(state.bg_color),
                        store: true,
                    },
                }],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_pass.texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: self.selected_pass == SelectedExtraPass::Depth,
                    }),
                    stencil_ops: None,
                }),
            };

            let mut render_pass = encoder.begin_render_pass(rp_desc);
            render_pass.set_pipeline(&self.pipeline);

            let selected_bind_group = match self.selected_image {
                SelectedImage::SanCheese => &self.diffuse1_bind_group,
                SelectedImage::Nnubes => &self.diffuse2_bind_group,
            };
            render_pass.set_bind_group(0, selected_bind_group, &[]);
            render_pass.set_bind_group(1, &self.uniform_bind_group, &[]);

            self.epic_mesh
                .render(&mut render_pass, Some(&self.instances_buffer));
        }

        match self.selected_pass {
            SelectedExtraPass::Depth => self.depth_pass.render(encoder, frame_view, state),
            SelectedExtraPass::None => Ok(()),
        }
    }

    fn resize(
        &mut self,
        device: &wgpu::Device,
        _queue: &wgpu::Queue,
        size: winit::dpi::PhysicalSize<u32>,
    ) {
        self.camera.aspect = size.width as f32 / size.height as f32;

        self.depth_pass.resize(device, size);
    }
}
