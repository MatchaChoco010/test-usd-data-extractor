use glam::Vec3;
use std::collections::HashMap;

#[derive(Debug)]
pub struct TimeCodeRange {
    pub start: i64,
    pub end: i64,
}

#[derive(Debug)]
pub struct Mesh {
    pub vertex_count: u32,
    pub vertex_buffer: Option<wgpu::Buffer>,
    pub model_buffer: wgpu::Buffer,
}

#[derive(Debug)]
pub struct DistantLight {
    pub direction: Vec3,
    pub intensity: f32,
    pub color: Vec3,
    pub angle: f32,
}

#[derive(Debug)]
pub struct SphereLight {
    pub is_spot: bool,
    pub position: Vec3,
    pub intensity: f32,
    pub color: Vec3,
    pub direction: Option<Vec3>,
    pub cone_angle: Option<f32>,
    pub cone_softness: Option<f32>,
}

#[derive(Debug)]
pub struct Camera {
    pub view_matrix: glam::Mat4,
    pub fovy: f32,
}
impl Camera {
    pub fn new() -> Self {
        Self {
            view_matrix: glam::Mat4::IDENTITY,
            fovy: 60.0_f32.to_radians(),
        }
    }
}

#[derive(Debug)]
pub struct Scene {
    pub range: Option<TimeCodeRange>,
    pub meshes: HashMap<String, Mesh>,
    pub sphere_lights: HashMap<String, SphereLight>,
    pub distant_lights: HashMap<String, DistantLight>,
    pub camera: Camera,
}
