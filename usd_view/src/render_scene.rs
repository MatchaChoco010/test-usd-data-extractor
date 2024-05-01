use glam::Vec3;
use std::collections::HashMap;
use std::sync::Arc;
use usd_data_extractor::*;
use wgpu::util::DeviceExt;

#[derive(Debug)]
struct RenderSceneMeshData {
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
    transform_buffer: wgpu::Buffer,
    vertex_buffer: Option<wgpu::Buffer>,
    vertex_count: u32,
    sub_mesh_indices: Vec<(wgpu::Buffer, u32)>,
}
impl RenderSceneMeshData {
    fn new(device: Arc<wgpu::Device>, queue: Arc<wgpu::Queue>) -> Self {
        let data = glam::Mat4::IDENTITY.to_cols_array();
        let transform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Model Buffer"),
            contents: bytemuck::cast_slice(&data),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        Self {
            device,
            queue,
            transform_buffer,
            vertex_buffer: None,
            vertex_count: 0,
            sub_mesh_indices: Vec::new(),
        }
    }

    fn update_transform_matrix(&mut self, transform_matrix: TransformMatrix) {
        let data = transform_matrix.matrix;
        self.queue.write_buffer(
            &self.transform_buffer,
            0,
            bytemuck::cast_slice(&data.to_cols_array()),
        );
    }

    fn update_mesh_data(&mut self, mesh: MeshData) {
        // vertex bufferの更新
        if self.vertex_buffer.is_none() {
            // vertex bufferが存在していない場合生成する
            let buffer = self
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Vertex Buffer"),
                    contents: bytemuck::cast_slice(&mesh.vertices),
                    usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                });
            self.vertex_buffer = Some(buffer);
            self.vertex_count = mesh.vertices.len() as u32;
        } else {
            // vertex bufferが存在している場合、頂点数が変わっていなければデータの更新のみ行い、
            // 頂点数が変わっていれば新しいバッファを生成しアップロードする
            if self.vertex_count == mesh.vertices.len() as u32 {
                self.queue.write_buffer(
                    &self.vertex_buffer.as_ref().unwrap(),
                    0,
                    bytemuck::cast_slice(&mesh.vertices),
                );
            } else {
                let buffer = self
                    .device
                    .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("Vertex Buffer"),
                        contents: bytemuck::cast_slice(&mesh.vertices),
                        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                    });
                self.vertex_buffer = Some(buffer);
                self.vertex_count = mesh.vertices.len() as u32;
            }
        }

        // sub mesh indicesの更新
        for (index, sub_mesh) in mesh.sub_meshes.iter().enumerate() {
            if self.sub_mesh_indices.len() <= index {
                // sub mesh indicesが存在していない場合生成する
                let buffer = self
                    .device
                    .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("Index Buffer"),
                        contents: bytemuck::cast_slice(&sub_mesh.indices),
                        usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
                    });
                self.sub_mesh_indices
                    .push((buffer, sub_mesh.indices.len() as u32));
            } else {
                // sub mesh indicesが存在している場合、要素数が変わっていなければデータの更新のみ行い、
                // 要素数が変わっていれば新しいバッファを生成しアップロードする
                let (buffer, count) = &mut self.sub_mesh_indices[index];
                if *count == sub_mesh.indices.len() as u32 {
                    self.queue
                        .write_buffer(buffer, 0, bytemuck::cast_slice(&sub_mesh.indices));
                } else {
                    let buf = self
                        .device
                        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: Some("Index Buffer"),
                            contents: bytemuck::cast_slice(&sub_mesh.indices),
                            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
                        });
                    *buffer = buf;
                    *count = sub_mesh.indices.len() as u32;
                }
            }
        }
    }
}

pub struct RenderMesh<'a> {
    pub transform_matrix_buffer: &'a wgpu::Buffer,
    pub vertex_buffer: &'a wgpu::Buffer,
    pub index_buffer: &'a wgpu::Buffer,
    pub index_count: u32,
}

#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct RenderDirectionalLight {
    pub direction: Vec3,
    pub intensity: f32,
    pub color: Vec3,
    pub _padding: u32,
}

#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct RenderPointLight {
    pub position: Vec3,
    pub intensity: f32,
    pub color: Vec3,
    pub _padding: u32,
}

#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct RenderSpotLight {
    pub position: Vec3,
    pub intensity: f32,
    pub direction: Vec3,
    pub angle: f32,
    pub color: Vec3,
    pub softness: f32,
}

#[derive(Debug)]
pub struct RenderScene {
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
    meshes: HashMap<String, RenderSceneMeshData>,
    sphere_lights: HashMap<String, SphereLight>,
    distant_lights: HashMap<String, DistantLight>,
    cameras: HashMap<String, Camera>,
    active_camera: Option<String>,
}
impl RenderScene {
    pub fn new(device: Arc<wgpu::Device>, queue: Arc<wgpu::Queue>) -> Self {
        Self {
            device,
            queue,
            meshes: HashMap::new(),
            sphere_lights: HashMap::new(),
            distant_lights: HashMap::new(),
            cameras: HashMap::new(),
            active_camera: None,
        }
    }

    // === Update data ===

    pub fn add_mesh(&mut self, name: String, transform_matrix: TransformMatrix, mesh: MeshData) {
        let mut mesh_data =
            RenderSceneMeshData::new(Arc::clone(&self.device), Arc::clone(&self.queue));
        mesh_data.update_transform_matrix(transform_matrix);
        mesh_data.update_mesh_data(mesh);
        self.meshes.insert(name, mesh_data);
    }

    pub fn remove_mesh(&mut self, name: String) {
        self.meshes.remove(&name);
    }

    pub fn update_mesh_transform_matrix(
        &mut self,
        name: String,
        transform_matrix: TransformMatrix,
    ) {
        if let Some(mesh) = self.meshes.get_mut(&name) {
            mesh.update_transform_matrix(transform_matrix);
        }
    }

    pub fn update_mesh_data(&mut self, name: String, mesh: MeshData) {
        if let Some(mesh_data) = self.meshes.get_mut(&name) {
            mesh_data.update_mesh_data(mesh);
        }
    }

    pub fn insert_sphere_light(&mut self, name: String, light: SphereLight) {
        self.sphere_lights.insert(name, light);
    }

    pub fn remove_sphere_light(&mut self, name: String) {
        self.sphere_lights.remove(&name);
    }

    pub fn insert_distant_light(&mut self, name: String, light: DistantLight) {
        self.distant_lights.insert(name, light);
    }

    pub fn remove_distant_light(&mut self, name: String) {
        self.distant_lights.remove(&name);
    }

    pub fn insert_camera(&mut self, name: String, camera: Camera) {
        self.cameras.insert(name, camera);
    }

    pub fn remove_camera(&mut self, name: String) {
        self.cameras.remove(&name);
    }

    pub fn set_active_camera_path(&mut self, name: Option<String>) {
        if let Some(name) = name {
            if self.cameras.get(&name).is_some() {
                self.active_camera = Some(name);
            } else {
                self.active_camera = None;
            }
        } else {
            self.active_camera = None;
        }
    }

    // === Get Render data ===

    pub fn get_render_meshes<'a>(&'a self) -> Vec<RenderMesh<'a>> {
        self.meshes
            .iter()
            .filter(|(_, mesh)| mesh.vertex_buffer.is_some())
            .flat_map(|(_, mesh)| {
                mesh.sub_mesh_indices
                    .iter()
                    .map(move |(index_buffer, count)| RenderMesh {
                        transform_matrix_buffer: &mesh.transform_buffer,
                        vertex_buffer: mesh.vertex_buffer.as_ref().unwrap(),
                        index_buffer,
                        index_count: *count,
                    })
            })
            .collect()
    }

    pub fn get_lights(
        &self,
    ) -> (
        Vec<RenderDirectionalLight>,
        Vec<RenderPointLight>,
        Vec<RenderSpotLight>,
    ) {
        let directional_lights = self
            .distant_lights
            .iter()
            .map(|(_, light)| RenderDirectionalLight {
                direction: light.direction,
                intensity: light.intensity,
                color: light.color,
                _padding: 0,
            })
            .collect();
        let point_lights = self
            .sphere_lights
            .iter()
            .filter_map(|(_, light)| {
                if light.is_spot {
                    None
                } else {
                    Some(RenderPointLight {
                        position: light.position,
                        intensity: light.intensity,
                        color: light.color,
                        _padding: 0,
                    })
                }
            })
            .collect();
        let spot_lights = self
            .sphere_lights
            .iter()
            .filter_map(|(_, light)| {
                if light.is_spot {
                    Some(RenderSpotLight {
                        position: light.position,
                        intensity: light.intensity,
                        direction: light.direction.unwrap(),
                        angle: light.cone_angle.unwrap(),
                        color: light.color,
                        softness: light.cone_softness.unwrap(),
                    })
                } else {
                    None
                }
            })
            .collect();
        (directional_lights, point_lights, spot_lights)
    }

    pub fn get_camera(&self) -> Camera {
        if let Some(name) = &self.active_camera {
            self.cameras.get(name).unwrap().clone()
        } else {
            Camera {
                eye: Vec3::new(0.0, 1.2, 5.0),
                dir: Vec3::new(0.0, 0.0, -1.0),
                fovy: 60.0_f32.to_radians(),
            }
        }
    }
}
