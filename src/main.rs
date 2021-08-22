use std::sync::Arc;

use buffer::StagingFactory;
use futures::executor::block_on;
use scenes::Scene;
use wgpu::{TextureViewDescriptor, TextureViewDimension};
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

mod buffer;
mod camera;
mod mesh;
mod scene;
mod texture;
mod vertex;

use crate::scene as scenes;

#[derive(Clone, Copy, Debug)]
pub(crate) enum CurrentDemo {
    Textured,
    Cameras,
    Instancing,
    ClownColors,
    Dima,
}

impl CurrentDemo {
    fn next(&mut self) {
        *self = match self {
            CurrentDemo::Textured => CurrentDemo::Cameras,
            CurrentDemo::Cameras => CurrentDemo::Instancing,
            CurrentDemo::Instancing => CurrentDemo::ClownColors,
            CurrentDemo::ClownColors => CurrentDemo::Dima,
            CurrentDemo::Dima => CurrentDemo::Textured,
        }
    }
}

pub(crate) struct GlobalState {
    bg_color: wgpu::Color,
}

impl Default for GlobalState {
    fn default() -> Self {
        GlobalState {
            bg_color: wgpu::Color {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 1.0,
            },
        }
    }
}

struct State {
    surface: wgpu::Surface,
    device: Arc<wgpu::Device>,
    queue: wgpu::Queue,
    sc_desc: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    user_state: GlobalState,
    current_pipeline: CurrentDemo,
    staging: StagingFactory,

    demo1: scenes::textured::TextureExampleScene,
    demo2: scenes::clown::ClownColorsScene,
    demo3: scenes::triangle::TriangleScene,
    demo4: scenes::camera::CameraScene,
    demo5: scenes::instancing::InstancesScene,
}

impl State {
    async fn new(window: &Window) -> Self {
        // Get the window's inner size
        let size = window.inner_size();

        // Get a handle to the graphics library
        let instance = wgpu::Instance::new(wgpu::Backends::PRIMARY);

        // Create a drawing surface for our window
        let surface = unsafe { instance.create_surface(window) };

        // Request an adapter (handle to GPU) for that surface
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
            })
            .await
            .unwrap();

        // From the adapter, request the corresponding device and queue
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("Main device descriptor"),
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::default(),
                },
                None,
            )
            .await
            .unwrap();

        let device = Arc::new(device);

        // Create the swap chain for our surface
        let sc_desc = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
        };

        //let swap_chain = device.create(&surface, &sc_desc);
        surface.configure(&device, &sc_desc);

        let user_state = GlobalState::default();

        let mut staging = StagingFactory::new(&device);

        let demo1 =
            scenes::textured::TextureExampleScene::new(&device, &queue, &sc_desc, &mut staging);
        let demo2 = scenes::clown::ClownColorsScene::new(&device, &queue, &sc_desc, &mut staging);
        let demo3 = scenes::triangle::TriangleScene::new(&device, &queue, &sc_desc, &mut staging);
        let demo4 = scenes::camera::CameraScene::new(&device, &queue, &sc_desc, &mut staging);
        let demo5 =
            scenes::instancing::InstancesScene::new(&device, &queue, &sc_desc, &mut staging);

        Self {
            surface,
            device,
            queue,
            sc_desc,
            size,
            user_state,
            staging,
            demo1,
            demo2,
            demo3,
            demo4,
            demo5,
            current_pipeline: CurrentDemo::Textured,
        }
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.size = new_size;
        self.sc_desc.width = new_size.width;
        self.sc_desc.height = new_size.height;
        self.surface.configure(&self.device, &self.sc_desc);

        match self.current_pipeline {
            CurrentDemo::Textured => self.demo1.resize(&self.device, &self.queue, new_size),
            CurrentDemo::ClownColors => self.demo2.resize(&self.device, &self.queue, new_size),
            CurrentDemo::Dima => self.demo3.resize(&self.device, &self.queue, new_size),
            CurrentDemo::Cameras => self.demo4.resize(&self.device, &self.queue, new_size),
            CurrentDemo::Instancing => self.demo5.resize(&self.device, &self.queue, new_size),
        }
    }

    fn input(&mut self, event: &WindowEvent) -> bool {
        let handled_input = match self.current_pipeline {
            CurrentDemo::Textured => self.demo1.input(event),
            CurrentDemo::ClownColors => self.demo2.input(event),
            CurrentDemo::Dima => self.demo3.input(event),
            CurrentDemo::Cameras => self.demo4.input(event),
            CurrentDemo::Instancing => self.demo5.input(event),
        };

        match event {
            WindowEvent::CursorMoved { position, .. } => {
                let color_x = position.x / (self.size.width as f64);
                let color_y = position.y / (self.size.height as f64);

                let bg_color = &mut self.user_state.bg_color;
                bg_color.r = color_x;
                bg_color.g = color_y;

                true
            }
            WindowEvent::KeyboardInput { input, .. } => {
                if let KeyboardInput {
                    state: ElementState::Pressed,
                    virtual_keycode: Some(VirtualKeyCode::Space),
                    ..
                } = input
                {
                    println!("Pressed spacebar");
                    self.current_pipeline.next();
                    println!("{:?}", self.current_pipeline);
                }

                true
            }
            _ => handled_input,
        }
    }

    fn update(&mut self) {
        match self.current_pipeline {
            CurrentDemo::Textured => self.demo1.update(&self.device, &self.queue),
            CurrentDemo::ClownColors => self.demo2.update(&self.device, &self.queue),
            CurrentDemo::Dima => self.demo3.update(&self.device, &self.queue),
            CurrentDemo::Cameras => self.demo4.update(&self.device, &self.queue),
            CurrentDemo::Instancing => self.demo5.update(&self.device, &self.queue),
        }
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        // Get the frame we are going to draw on
        let frame = self.surface.get_current_frame()?.output;

        let texture_view = frame.texture.create_view(&TextureViewDescriptor {
            label: Some("Main Texture View"),
            format: Some(self.sc_desc.format),
            dimension: Some(TextureViewDimension::D2),
            aspect: wgpu::TextureAspect::All,
            base_mip_level: 0,
            mip_level_count: None,
            base_array_layer: 0,
            array_layer_count: None,
        });

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Command Encoder"),
            });

        match self.current_pipeline {
            CurrentDemo::Textured => {
                self.demo1
                    .render(&mut encoder, &texture_view, &self.user_state, &self.staging)
            }
            CurrentDemo::ClownColors => {
                self.demo2
                    .render(&mut encoder, &texture_view, &self.user_state, &self.staging)
            }
            CurrentDemo::Dima => {
                self.demo3
                    .render(&mut encoder, &texture_view, &self.user_state, &self.staging)
            }
            CurrentDemo::Cameras => {
                self.demo4
                    .render(&mut encoder, &texture_view, &self.user_state, &self.staging)
            }
            CurrentDemo::Instancing => {
                self.demo5
                    .render(&mut encoder, &texture_view, &self.user_state, &self.staging)
            }
        }?;

        self.staging.submit_all();
        self.queue.submit(std::iter::once(encoder.finish()));
        self.staging.recall_all();

        Ok(())
    }
}

fn main() {
    env_logger::init();

    // Create winit event loop
    let event_loop = EventLoop::new();

    // Create a window
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    // Initialize our graphics state
    let mut state = block_on(State::new(&window));

    // Run the event loop
    event_loop.run(move |event, _, control_flow| match event {
        Event::RedrawRequested(_) => {
            state.update();
            match state.render() {
                Ok(_) => {}
                // Recreate the swap_chain if lost
                Err(wgpu::SurfaceError::Lost) => state.resize(state.size),
                // The system is out of memory, we should probably quit
                Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                // All other errors (Outdated, Timeout) should be resolved by the next frame
                Err(e) => eprintln!("{:?}", e),
            }
        }
        Event::MainEventsCleared => {
            // RedrawRequested will only trigger once, unless we manually
            // request it.
            window.request_redraw();
        }
        Event::WindowEvent {
            ref event,
            window_id,
            // Only match on our own window
        } if window_id == window.id() => {
            // We handle the input received, and if it's not
            // input, we keep propagating
            if !state.input(event) {
                match event {
                    // If the user changes the window size, update the swap chain
                    WindowEvent::Resized(physical_size) => {
                        state.resize(*physical_size);
                    }
                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        state.resize(**new_inner_size);
                    }
                    // If the user wants to exit, let them
                    WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                    WindowEvent::KeyboardInput { input, .. } => {
                        if let KeyboardInput {
                            state: ElementState::Pressed,
                            virtual_keycode: Some(VirtualKeyCode::Escape),
                            ..
                        } = input
                        {
                            *control_flow = ControlFlow::Exit
                        }
                    }
                    _ => {}
                }
            }
        }
        _ => {}
    });
}

mod test {}
