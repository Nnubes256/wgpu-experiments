use std::ops::Range;

use cgmath::{Matrix4, Quaternion, SquareMatrix, Vector3};
use wgpu::util::DeviceExt;

use crate::{
    buffer::IndexedVertexBuffer,
    vertex::{Descriptable, VertexBufferable},
};

#[derive(Debug)]
pub struct Transform {
    translation: Vector3<f32>,
    rotation: Quaternion<f32>,
    scale: Vector3<f32>,
    mat: Matrix4<f32>,
}

impl Transform {
    pub fn new(translation: Vector3<f32>, rotation: Quaternion<f32>, scale: Vector3<f32>) -> Self {
        let mut myself = Self {
            translation,
            rotation,
            scale,
            mat: Matrix4::identity(),
        };
        myself.regenerate_model_matrix();
        myself
    }

    pub fn translation(&self) -> &Vector3<f32> {
        &self.translation
    }

    pub fn set_translation(&mut self, f: impl FnOnce(&mut Vector3<f32>)) {
        f(&mut self.translation);
        self.regenerate_model_matrix();
    }

    pub unsafe fn translation_mut(&mut self) -> &mut Vector3<f32> {
        &mut self.translation
    }

    pub fn rotation(&self) -> &Quaternion<f32> {
        &self.rotation
    }

    pub fn set_rotation(&mut self, f: impl FnOnce(&mut Quaternion<f32>)) {
        f(&mut self.rotation);
        self.regenerate_model_matrix();
    }

    pub unsafe fn rotation_mut(&mut self) -> &mut Quaternion<f32> {
        &mut self.rotation
    }

    pub fn scale(&self) -> &Vector3<f32> {
        &self.scale
    }

    pub fn set_scale(&mut self, f: impl FnOnce(&mut Vector3<f32>)) {
        f(&mut self.scale);
        self.regenerate_model_matrix();
    }

    pub unsafe fn scale_mut(&mut self) -> &mut Vector3<f32> {
        &mut self.scale
    }

    pub fn regenerate_model_matrix(&mut self) {
        let t = self.translation;
        let s = self.scale;
        self.mat = Matrix4::from_nonuniform_scale(s.x, s.y, s.z);
        self.mat.w.x = t.x;
        self.mat.w.y = t.y;
        self.mat.w.z = t.z;
        let r_mat: Matrix4<f32> = self.rotation.into();
        self.mat = self.mat * r_mat;
    }

    pub fn model_matrix(&self) -> &Matrix4<f32> {
        &self.mat
    }

    pub fn uniform_matrix2(&self) -> &[[f32; 4]; 4] {
        self.mat.as_ref()
    }

    pub fn uniform_matrix(&self) -> [[f32; 4]; 4] {
        self.mat.into()
    }

    pub fn as_buffer(&self, device: &wgpu::Device, label: Option<&str>) -> wgpu::Buffer {
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label,
            contents: bytemuck::bytes_of(self.uniform_matrix2()),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        })
    }
}

/*impl Uniformable for Transform {
    type Uniform = [[f32; 4]; 4];

    fn into_uniform(&self) -> Self::Uniform {
        self.mat.into()
    }
}*/

#[macro_export]
macro_rules! transform {
    {
        t: [$t1:expr, $t2:expr, $t3:expr],
        r: [$r1:expr, $r2:expr, $r3:expr],
        s: [$s1:expr, $s2:expr, $s3:expr]
    } => {
        {
            use cgmath::{Vector3, Quaternion, Euler, Deg};
            Transform::new(
                Vector3::new($t1, $t2, $t3),
                Quaternion::from(Euler { x: Deg($r1), y: Deg($r2), z: Deg($r3)}),
                Vector3::new($s1, $s2, $s3)
            )
        }
    }
}

pub struct MeshRenderData {
    next_vertex_idx: u32,
}

pub struct OldMesh<T: VertexBufferable + Descriptable> {
    data: IndexedVertexBuffer<T>,
    transform: Transform,
}

impl<T: VertexBufferable + Descriptable> OldMesh<T> {
    pub fn new(data: IndexedVertexBuffer<T>, transform: Transform) -> Self {
        Self { data, transform }
    }

    pub fn transform(&self) -> &Transform {
        &self.transform
    }

    pub fn transform_mut(&mut self) -> &mut Transform {
        &mut self.transform
    }

    pub fn render<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>, instances: Range<u32>) {
        render_pass.set_vertex_buffer(0, self.data.vertices.slice(..));
        render_pass.set_index_buffer(self.data.indices.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.draw_indexed(0..self.data.num_indices, 0, instances)
    }
}
