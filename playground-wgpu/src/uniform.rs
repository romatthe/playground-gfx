use cgmath::{Matrix4, SquareMatrix};
use crate::camera::Camera;

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Uniforms {
    view_proj: Matrix4<f32>,
}

unsafe impl bytemuck::Pod for Uniforms {}

unsafe impl bytemuck::Zeroable for Uniforms {}

impl Uniforms {
    pub fn new() -> Self {
        Self {
            view_proj: Matrix4::identity()
        }
    }

    pub fn update_view_proj(&mut self, camera: &Camera) {
        self.view_proj = camera.build_view_projection_matrix();
    }
}