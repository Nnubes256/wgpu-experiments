use cgmath::Vector3;
use winit::event::{ElementState, KeyboardInput, VirtualKeyCode, WindowEvent};

use crate::buffer::OldUniform;

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
);

pub struct Camera {
    pub eye: cgmath::Point3<f32>,
    pub target: cgmath::Point3<f32>,
    pub up: cgmath::Vector3<f32>,
    pub aspect: f32,
    pub fovy: f32,
    pub znear: f32,
    pub zfar: f32,
}

impl Camera {
    pub fn build_view_projection_matrix(&self) -> cgmath::Matrix4<f32> {
        let view = cgmath::Matrix4::look_at(self.eye, self.target, self.up);

        let projection =
            cgmath::perspective(cgmath::Deg(self.fovy), self.aspect, self.znear, self.zfar);

        OPENGL_TO_WGPU_MATRIX * projection * view
    }
}

pub struct CameraController {
    speed: f32,
    is_up_pressed: bool,
    is_down_pressed: bool,
    is_forward_pressed: bool,
    is_backward_pressed: bool,
    is_left_pressed: bool,
    is_right_pressed: bool,
}

impl CameraController {
    pub fn new(speed: f32) -> Self {
        Self {
            speed,
            is_up_pressed: false,
            is_down_pressed: false,
            is_forward_pressed: false,
            is_backward_pressed: false,
            is_left_pressed: false,
            is_right_pressed: false,
        }
    }

    pub fn input(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        state,
                        virtual_keycode: Some(keycode),
                        ..
                    },
                ..
            } => {
                let is_pressed = *state == ElementState::Pressed;
                match keycode {
                    VirtualKeyCode::Q => {
                        self.is_up_pressed = is_pressed;
                        true
                    }
                    VirtualKeyCode::E => {
                        self.is_down_pressed = is_pressed;
                        true
                    }
                    VirtualKeyCode::W | VirtualKeyCode::Up => {
                        self.is_forward_pressed = is_pressed;
                        true
                    }
                    VirtualKeyCode::A | VirtualKeyCode::Left => {
                        self.is_left_pressed = is_pressed;
                        true
                    }
                    VirtualKeyCode::S | VirtualKeyCode::Down => {
                        self.is_backward_pressed = is_pressed;
                        true
                    }
                    VirtualKeyCode::D | VirtualKeyCode::Right => {
                        self.is_right_pressed = is_pressed;
                        true
                    }
                    _ => false,
                }
            }
            _ => false,
        }
    }

    pub fn update(&self, camera: &mut Camera) {
        use cgmath::InnerSpace;

        // Get the forward vector, and normalize it
        let forward: Vector3<f32> = camera.target - camera.eye;
        let forward_norm = forward.normalize();

        // Get its length
        let forward_mag = forward.magnitude();

        // Forward/backwards movement
        // Forward movement is limited in order to avoid near clipping issues
        if self.is_forward_pressed && forward_mag > self.speed {
            camera.eye += forward_norm * self.speed;
        }
        if self.is_backward_pressed {
            camera.eye -= forward_norm * self.speed;
        }

        // Redo radius calc in case the directional keys are pressed
        let forward = camera.target - camera.eye;
        let forward_mag = forward.magnitude();

        // Up/down movement
        camera.eye = match (self.is_up_pressed, self.is_down_pressed) {
            (true, true) | (false, false) => camera.eye,
            (true, false) => {
                // Rescale the distance between the target and eye so
                // that it doesn't change. The eye therefore still
                // lies on the circle made by the target and eye.
                camera.target - (forward + camera.up * self.speed).normalize() * forward_mag
            }
            (false, true) => {
                camera.target - (forward - camera.up * self.speed).normalize() * forward_mag
            }
        };

        // Get the right-facing vector as the cross product
        // of the forward and up vectors
        let right = forward_norm.cross(camera.up);

        // Left/right movement
        camera.eye = match (self.is_left_pressed, self.is_right_pressed) {
            (true, true) | (false, false) => camera.eye,
            (true, false) => {
                camera.target - (forward + right * self.speed).normalize() * forward_mag
            }
            (false, true) => {
                camera.target - (forward - right * self.speed).normalize() * forward_mag
            }
        };
    }
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    pub view_proj: [[f32; 4]; 4],
}

impl Default for CameraUniform {
    fn default() -> Self {
        use cgmath::SquareMatrix;
        Self {
            view_proj: cgmath::Matrix4::identity().into(),
        }
    }
}

impl OldUniform for CameraUniform {}

impl CameraUniform {
    pub fn update(&mut self, camera: &Camera) {
        self.view_proj = camera.build_view_projection_matrix().into();
    }
}
