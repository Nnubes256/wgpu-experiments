use winit::event::WindowEvent;

use crate::{buffer::StagingFactory, GlobalState};

pub mod camera;
pub mod clown;
pub mod instancing;
pub mod textured;
pub mod triangle;

pub(crate) trait Scene {
    fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        sc: &wgpu::SurfaceConfiguration,
        staging: &mut StagingFactory,
    ) -> Self;
    fn input(&mut self, event: &WindowEvent) -> bool;
    fn update(&mut self, device: &wgpu::Device, queue: &wgpu::Queue);
    fn render(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        frame_view: &wgpu::TextureView,
        state: &GlobalState,
        staging: &StagingFactory,
    ) -> Result<(), wgpu::SurfaceError>;
    fn recall(&mut self) {}
    fn resize(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        size: winit::dpi::PhysicalSize<u32>,
    );
}
