use glam::Vec3;
use image::GenericImageView;
use std::collections::HashMap;
use std::sync::Arc;
use usd_data_extractor::*;
use wgpu::util::DeviceExt;

use crate::renderer::{RenderDirectionalLight, RenderPointLight, RenderSpotLight};

#[derive(Debug)]
struct RenderSubMeshData {
    index_buffer: wgpu::Buffer,
    indices_count: u32,
    material_buffer: wgpu::Buffer,
    material_path: Option<String>,
}

#[derive(Debug)]
struct RenderMeshData {
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
    transform_buffer: wgpu::Buffer,
    vertex_buffer: Option<wgpu::Buffer>,
    vertex_count: u32,
    sub_meshes: Vec<RenderSubMeshData>,
}
impl RenderMeshData {
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
            sub_meshes: Vec::new(),
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

        // sub meshesの更新
        for (index, sub_mesh) in mesh.sub_meshes.iter().enumerate() {
            if self.sub_meshes.len() <= index {
                // sub meshの要素数が足りない場合生成する
                let index_buffer =
                    self.device
                        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: Some("Index Buffer"),
                            contents: bytemuck::cast_slice(&sub_mesh.indices),
                            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
                        });
                let material_buffer =
                    self.device
                        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: Some("Material Buffer"),
                            contents: bytemuck::cast_slice(&[crate::renderer::Material::default()]),
                            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                        });
                self.sub_meshes.push(RenderSubMeshData {
                    index_buffer,
                    indices_count: sub_mesh.indices.len() as u32,
                    material_buffer,
                    material_path: sub_mesh.material.clone(),
                });
            } else {
                // 既存のsub meshを更新する
                let RenderSubMeshData {
                    index_buffer,
                    indices_count,
                    material_path,
                    ..
                } = &mut self.sub_meshes[index];

                // index bufferの更新
                // sub meshのindicesの要素数が変わっていなければデータの更新のみ行い、
                // 要素数が変わっていれば新しいバッファを生成しアップロードする
                if *indices_count == sub_mesh.indices.len() as u32 {
                    self.queue.write_buffer(
                        index_buffer,
                        0,
                        bytemuck::cast_slice(&sub_mesh.indices),
                    );
                } else {
                    let buf = self
                        .device
                        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: Some("Index Buffer"),
                            contents: bytemuck::cast_slice(&sub_mesh.indices),
                            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
                        });
                    *index_buffer = buf;
                    *indices_count = sub_mesh.indices.len() as u32;
                }

                // material pathの更新
                if let Some(sub_mesh_material_path) = &sub_mesh.material {
                    *material_path = Some(sub_mesh_material_path.clone());
                } else {
                    *material_path = None;
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
    pub material_buffer: &'a wgpu::Buffer,
    pub diffuse_texture: &'a wgpu::TextureView,
    pub diffuse_sampler: &'a wgpu::Sampler,
    pub emissive_texture: &'a wgpu::TextureView,
    pub emissive_sampler: &'a wgpu::Sampler,
    pub metallic_texture: &'a wgpu::TextureView,
    pub metallic_sampler: &'a wgpu::Sampler,
    pub roughness_texture: &'a wgpu::TextureView,
    pub roughness_sampler: &'a wgpu::Sampler,
    pub normal_texture: &'a wgpu::TextureView,
    pub normal_sampler: &'a wgpu::Sampler,
    pub opacity_texture: &'a wgpu::TextureView,
    pub opacity_sampler: &'a wgpu::Sampler,
}

#[derive(Debug)]
struct MaterialData {
    diffuse: Vec3,
    emissive: Vec3,
    metallic: f32,
    roughness: f32,
    opacity: f32,
    diffuse_texture: Option<String>,
    emissive_texture: Option<String>,
    metallic_texture: Option<String>,
    roughness_texture: Option<String>,
    normal_texture: Option<String>,
    opacity_texture: Option<String>,
}

#[derive(Debug)]
struct TextureDataItem {
    _texture: wgpu::Texture,
    view: wgpu::TextureView,
    sampler: wgpu::Sampler,
    reference_count: u32,
}

enum TextureType {
    Diffuse,
    Emissive,
    Metallic,
    Roughness,
    Normal,
    Opacity,
}

#[derive(Debug)]
struct TextureData {
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
    data: HashMap<String, TextureDataItem>,
}
impl TextureData {
    fn new(device: Arc<wgpu::Device>, queue: Arc<wgpu::Queue>) -> Self {
        Self {
            device,
            queue,
            data: HashMap::new(),
        }
    }

    fn reference_count_decrement(&mut self, path: &String) {
        if let Some(item) = self.data.get_mut(path) {
            item.reference_count -= 1;
            if item.reference_count == 0 {
                self.data.remove(path);
            }
        }
    }

    fn reference_count_increment_or_load(&mut self, path: &String, texture_type: TextureType) {
        if let Some(item) = self.data.get_mut(path) {
            item.reference_count += 1;
        } else {
            // テクスチャの読み込み処理
            let image = image::open(path).unwrap();
            let format = match texture_type {
                TextureType::Diffuse => wgpu::TextureFormat::Rgba8Unorm,
                TextureType::Emissive => wgpu::TextureFormat::Rgba8Unorm,
                TextureType::Metallic => wgpu::TextureFormat::R8Unorm,
                TextureType::Roughness => wgpu::TextureFormat::R8Unorm,
                TextureType::Normal => wgpu::TextureFormat::Rgba8Unorm,
                TextureType::Opacity => wgpu::TextureFormat::R8Unorm,
            };
            let data = match texture_type {
                TextureType::Diffuse => image.to_rgba8().into_raw(),
                TextureType::Emissive => image.to_rgba8().into_raw(),
                TextureType::Metallic => image.to_luma8().into_raw(),
                TextureType::Roughness => image.to_luma8().into_raw(),
                TextureType::Normal => image.to_rgba8().into_raw(),
                TextureType::Opacity => image
                    .to_rgba8()
                    .enumerate_pixels()
                    .map(|(_, _, p)| p.0[3])
                    .collect(),
            };
            let size = image.dimensions();
            let bytes_per_row = match texture_type {
                TextureType::Diffuse => 4 * size.0,
                TextureType::Emissive => 4 * size.0,
                TextureType::Metallic => size.0,
                TextureType::Roughness => size.0,
                TextureType::Normal => 4 * size.0,
                TextureType::Opacity => size.0,
            };
            let rows_per_image = size.1;
            let texture_size = wgpu::Extent3d {
                width: size.0,
                height: size.1,
                depth_or_array_layers: 1,
            };
            let texture = self.device.create_texture(&wgpu::TextureDescriptor {
                size: texture_size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                label: None,
                view_formats: &[],
            });
            self.queue.write_texture(
                wgpu::ImageCopyTexture {
                    texture: &texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                &data,
                wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(bytes_per_row),
                    rows_per_image: Some(rows_per_image),
                },
                texture_size,
            );
            let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
            let sampler = self.device.create_sampler(&wgpu::SamplerDescriptor {
                address_mode_u: wgpu::AddressMode::ClampToEdge,
                address_mode_v: wgpu::AddressMode::ClampToEdge,
                address_mode_w: wgpu::AddressMode::ClampToEdge,
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Nearest,
                mipmap_filter: wgpu::FilterMode::Nearest,
                ..Default::default()
            });
            let texture_data = TextureDataItem {
                _texture: texture,
                view,
                sampler,
                reference_count: 1,
            };
            self.data.insert(path.clone(), texture_data);
        }
    }

    fn update_material_texture(
        &mut self,
        prev_material_data: Option<&MaterialData>,
        new_material_data: &MaterialData,
    ) {
        // diffuse_textureの読み込み
        if prev_material_data.is_none()
            || prev_material_data.unwrap().diffuse_texture != new_material_data.diffuse_texture
        {
            if let Some(prev_material_data) = prev_material_data {
                if let Some(path) = &prev_material_data.diffuse_texture {
                    self.reference_count_decrement(path);
                }
            }
            if let Some(path) = &new_material_data.diffuse_texture {
                self.reference_count_increment_or_load(path, TextureType::Diffuse);
            }
        }

        // emissive_textureの読み込み
        if prev_material_data.is_none()
            || prev_material_data.unwrap().emissive_texture != new_material_data.emissive_texture
        {
            if let Some(prev_material_data) = prev_material_data {
                if let Some(path) = &prev_material_data.emissive_texture {
                    self.reference_count_decrement(path);
                }
            }
            if let Some(path) = &new_material_data.emissive_texture {
                self.reference_count_increment_or_load(path, TextureType::Emissive);
            }
        }

        // metallic_textureの読み込み
        if prev_material_data.is_none()
            || prev_material_data.unwrap().metallic_texture != new_material_data.metallic_texture
        {
            if let Some(prev_material_data) = prev_material_data {
                if let Some(path) = &prev_material_data.metallic_texture {
                    self.reference_count_decrement(path);
                }
            }
            if let Some(path) = &new_material_data.metallic_texture {
                self.reference_count_increment_or_load(path, TextureType::Metallic);
            }
        }

        // roughness_textureの読み込み
        if prev_material_data.is_none()
            || prev_material_data.unwrap().roughness_texture != new_material_data.roughness_texture
        {
            if let Some(prev_material_data) = prev_material_data {
                if let Some(path) = &prev_material_data.roughness_texture {
                    self.reference_count_decrement(path);
                }
            }
            if let Some(path) = &new_material_data.roughness_texture {
                self.reference_count_increment_or_load(path, TextureType::Roughness);
            }
        }

        // normal_textureの読み込み
        if prev_material_data.is_none()
            || prev_material_data.unwrap().normal_texture != new_material_data.normal_texture
        {
            if let Some(prev_material_data) = prev_material_data {
                if let Some(path) = &prev_material_data.normal_texture {
                    self.reference_count_decrement(path);
                }
            }
            if let Some(path) = &new_material_data.normal_texture {
                self.reference_count_increment_or_load(path, TextureType::Normal);
            }
        }

        // opacity_textureの読み込み
        if prev_material_data.is_none()
            || prev_material_data.unwrap().opacity_texture != new_material_data.opacity_texture
        {
            if let Some(prev_material_data) = prev_material_data {
                if let Some(path) = &prev_material_data.opacity_texture {
                    self.reference_count_decrement(path);
                }
            }
            if let Some(path) = &new_material_data.opacity_texture {
                self.reference_count_increment_or_load(path, TextureType::Opacity);
            }
        }
    }

    fn remove_material_texture(&mut self, remove_material_data: MaterialData) {
        if let Some(path) = &remove_material_data.diffuse_texture {
            self.reference_count_decrement(path);
        }
        if let Some(path) = &remove_material_data.emissive_texture {
            self.reference_count_decrement(path);
        }
        if let Some(path) = &remove_material_data.metallic_texture {
            self.reference_count_decrement(path);
        }
        if let Some(path) = &remove_material_data.roughness_texture {
            self.reference_count_decrement(path);
        }
        if let Some(path) = &remove_material_data.normal_texture {
            self.reference_count_decrement(path);
        }
        if let Some(path) = &remove_material_data.opacity_texture {
            self.reference_count_decrement(path);
        }
    }
}

#[derive(Debug)]
pub struct RenderScene {
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
    meshes: HashMap<String, RenderMeshData>,
    sphere_lights: HashMap<String, SphereLight>,
    distant_lights: HashMap<String, DistantLight>,
    cameras: HashMap<String, Camera>,
    active_camera: Option<String>,
    materials: HashMap<String, MaterialData>,
    textures: TextureData,
    _dummy_texture: wgpu::Texture,
    dummy_texture_view: wgpu::TextureView,
    dummy_sampler: wgpu::Sampler,
}
impl RenderScene {
    pub fn new(device: Arc<wgpu::Device>, queue: Arc<wgpu::Queue>) -> Self {
        let textures = TextureData::new(Arc::clone(&device), Arc::clone(&queue));

        let dummy_texture_size = wgpu::Extent3d {
            width: 1,
            height: 1,
            depth_or_array_layers: 1,
        };
        let dummy_texture = device.create_texture(&wgpu::TextureDescriptor {
            size: dummy_texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            label: None,
            view_formats: &[],
        });
        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &dummy_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &[255, 255, 255, 255],
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4),
                rows_per_image: Some(1),
            },
            dummy_texture_size,
        );
        let dummy_texture_view = dummy_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let dummy_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        Self {
            device,
            queue,
            meshes: HashMap::new(),
            sphere_lights: HashMap::new(),
            distant_lights: HashMap::new(),
            cameras: HashMap::new(),
            active_camera: None,
            materials: HashMap::new(),
            textures,
            _dummy_texture: dummy_texture,
            dummy_texture_view,
            dummy_sampler,
        }
    }

    // === Update data ===

    pub fn add_mesh(&mut self, name: String, transform_matrix: TransformMatrix, mesh: MeshData) {
        let mut mesh_data = RenderMeshData::new(Arc::clone(&self.device), Arc::clone(&self.queue));
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

    pub fn insert_material(&mut self, path: String, material: Material) {
        let new_material = MaterialData {
            diffuse: material.diffuse_color,
            emissive: material.emissive,
            metallic: material.metallic,
            roughness: material.roughness,
            opacity: material.opacity,
            diffuse_texture: material.diffuse_texture,
            emissive_texture: material.emissive_texture,
            metallic_texture: material.metallic_texture,
            roughness_texture: material.roughness_texture,
            normal_texture: material.normal_texture,
            opacity_texture: material.opacity_texture,
        };
        let prev_material = self.materials.get(&path);
        self.textures
            .update_material_texture(prev_material, &new_material);
        self.materials.insert(path, new_material);
    }

    pub fn remove_material(&mut self, name: String) {
        if let Some(material) = self.materials.remove(&name) {
            self.textures.remove_material_texture(material);
        }
    }

    // === Get Render data ===

    pub fn get_meshes<'a>(&'a self) -> Vec<RenderMesh<'a>> {
        self.meshes
            .iter()
            .filter(|(_, mesh)| mesh.vertex_buffer.is_some())
            .flat_map(|(_, mesh)| {
                mesh.sub_meshes.iter().map(move |sub_mesh| {
                    let RenderSubMeshData {
                        index_buffer,
                        indices_count: count,
                        material_path,
                        material_buffer,
                    } = sub_mesh;

                    if let Some(material_path) = material_path {
                        if let Some(material) = self.materials.get(material_path) {
                            let diffuse_texture = material
                                .diffuse_texture
                                .as_ref()
                                .and_then(|path| {
                                    self.textures.data.get(path).map(|item| &item.view)
                                })
                                .unwrap_or(&self.dummy_texture_view);
                            let diffuse_sampler = material
                                .diffuse_texture
                                .as_ref()
                                .and_then(|path| {
                                    self.textures.data.get(path).map(|item| &item.sampler)
                                })
                                .unwrap_or(&self.dummy_sampler);
                            let emissive_texture = material
                                .emissive_texture
                                .as_ref()
                                .and_then(|path| {
                                    self.textures.data.get(path).map(|item| &item.view)
                                })
                                .unwrap_or(&self.dummy_texture_view);
                            let emissive_sampler = material
                                .emissive_texture
                                .as_ref()
                                .and_then(|path| {
                                    self.textures.data.get(path).map(|item| &item.sampler)
                                })
                                .unwrap_or(&self.dummy_sampler);
                            let metallic_texture = material
                                .metallic_texture
                                .as_ref()
                                .and_then(|path| {
                                    self.textures.data.get(path).map(|item| &item.view)
                                })
                                .unwrap_or(&self.dummy_texture_view);
                            let metallic_sampler = material
                                .metallic_texture
                                .as_ref()
                                .and_then(|path| {
                                    self.textures.data.get(path).map(|item| &item.sampler)
                                })
                                .unwrap_or(&self.dummy_sampler);
                            let roughness_texture = material
                                .roughness_texture
                                .as_ref()
                                .and_then(|path| {
                                    self.textures.data.get(path).map(|item| &item.view)
                                })
                                .unwrap_or(&self.dummy_texture_view);
                            let roughness_sampler = material
                                .roughness_texture
                                .as_ref()
                                .and_then(|path| {
                                    self.textures.data.get(path).map(|item| &item.sampler)
                                })
                                .unwrap_or(&self.dummy_sampler);
                            let normal_texture = material
                                .normal_texture
                                .as_ref()
                                .and_then(|path| {
                                    self.textures.data.get(path).map(|item| &item.view)
                                })
                                .unwrap_or(&self.dummy_texture_view);
                            let normal_sampler = material
                                .normal_texture
                                .as_ref()
                                .and_then(|path| {
                                    self.textures.data.get(path).map(|item| &item.sampler)
                                })
                                .unwrap_or(&self.dummy_sampler);
                            let opacity_texture = material
                                .opacity_texture
                                .as_ref()
                                .and_then(|path| {
                                    self.textures.data.get(path).map(|item| &item.view)
                                })
                                .unwrap_or(&self.dummy_texture_view);
                            let opacity_sampler = material
                                .opacity_texture
                                .as_ref()
                                .and_then(|path| {
                                    self.textures.data.get(path).map(|item| &item.sampler)
                                })
                                .unwrap_or(&self.dummy_sampler);

                            let material_buffer_data = crate::renderer::Material {
                                base_color: material.diffuse,
                                emissive: material.emissive,
                                metallic: material.metallic,
                                roughness: material.roughness,
                                opacity: material.opacity,
                                base_color_texture: material
                                    .diffuse_texture
                                    .as_ref()
                                    .map_or(0, |_| 1),
                                emissive_texture: material
                                    .emissive_texture
                                    .as_ref()
                                    .map_or(0, |_| 1),
                                metallic_texture: material
                                    .metallic_texture
                                    .as_ref()
                                    .map_or(0, |_| 1),
                                roughness_texture: material
                                    .roughness_texture
                                    .as_ref()
                                    .map_or(0, |_| 1),
                                normal_texture: material.normal_texture.as_ref().map_or(0, |_| 1),
                                opacity_texture: material.opacity_texture.as_ref().map_or(0, |_| 1),
                                _padding: 0,
                            };
                            self.queue.write_buffer(
                                material_buffer,
                                0,
                                bytemuck::cast_slice(&[material_buffer_data]),
                            );

                            return RenderMesh {
                                transform_matrix_buffer: &mesh.transform_buffer,
                                vertex_buffer: mesh.vertex_buffer.as_ref().unwrap(),
                                index_buffer,
                                index_count: *count,
                                material_buffer,
                                diffuse_texture,
                                diffuse_sampler,
                                emissive_texture,
                                emissive_sampler,
                                metallic_texture,
                                metallic_sampler,
                                roughness_texture,
                                roughness_sampler,
                                normal_texture,
                                normal_sampler,
                                opacity_texture,
                                opacity_sampler,
                            };
                        }
                    }

                    let material_buffer_data = crate::renderer::Material {
                        base_color: Vec3::ONE * 0.18,
                        emissive: Vec3::ZERO,
                        metallic: 0.0,
                        roughness: 1.0,
                        opacity: 1.0,
                        base_color_texture: 0,
                        emissive_texture: 0,
                        metallic_texture: 0,
                        roughness_texture: 0,
                        normal_texture: 0,
                        opacity_texture: 0,
                        _padding: 0,
                    };
                    self.queue.write_buffer(
                        material_buffer,
                        0,
                        bytemuck::cast_slice(&[material_buffer_data]),
                    );

                    RenderMesh {
                        transform_matrix_buffer: &mesh.transform_buffer,
                        vertex_buffer: mesh.vertex_buffer.as_ref().unwrap(),
                        index_buffer,
                        index_count: *count,
                        material_buffer,
                        diffuse_texture: &self.dummy_texture_view,
                        diffuse_sampler: &self.dummy_sampler,
                        emissive_texture: &self.dummy_texture_view,
                        emissive_sampler: &self.dummy_sampler,
                        metallic_texture: &self.dummy_texture_view,
                        metallic_sampler: &self.dummy_sampler,
                        roughness_texture: &self.dummy_texture_view,
                        roughness_sampler: &self.dummy_sampler,
                        normal_texture: &self.dummy_texture_view,
                        normal_sampler: &self.dummy_sampler,
                        opacity_texture: &self.dummy_texture_view,
                        opacity_sampler: &self.dummy_sampler,
                    }
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
