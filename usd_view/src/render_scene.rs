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

#[derive(Debug)]
pub struct RenderScene {
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
    meshes: HashMap<String, RenderSceneMeshData>,
}
impl RenderScene {
    pub fn new(device: Arc<wgpu::Device>, queue: Arc<wgpu::Queue>) -> Self {
        Self {
            device,
            queue,
            meshes: HashMap::new(),
        }
    }

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
}