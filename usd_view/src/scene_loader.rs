use glam::{Vec2, Vec3};
use std::{
    collections::HashMap,
    sync::{
        mpsc::{channel, Receiver, Sender},
        Arc, Mutex,
    },
};
use usd_data_extractor::*;

use crate::renderer::{Model, Vertex};

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
pub struct Scene {
    pub range: Option<TimeCodeRange>,
    pub meshes: HashMap<String, Mesh>,
    pub sphere_lights: HashMap<String, SphereLight>,
    pub distant_lights: HashMap<String, DistantLight>,
}

#[derive(Debug)]
struct UsdSceneMesh {
    dirty_transform: bool,
    transform_matrix: Option<[f32; 16]>,
    dirty_mesh: bool,
    left_handed: bool,
    points_data: Option<Vec<f32>>,
    points_interpolation: Option<Interpolation>,
    normals_data: Option<Vec<f32>>,
    normals_interpolation: Option<Interpolation>,
    uvs_data: Option<Vec<f32>>,
    uvs_interpolation: Option<Interpolation>,
    face_vertex_indices: Option<Vec<u64>>,
    face_vertex_counts: Option<Vec<u32>>,
}

#[derive(Debug)]
struct UsdSceneDistantLight {
    dirty_transform: bool,
    transform_matrix: Option<[f32; 16]>,
    dirty_params: bool,
    intensity: Option<f32>,
    color: Option<[f32; 3]>,
    angle: Option<f32>,
}

#[derive(Debug)]
struct UsdSceneSphereLight {
    dirty_transform: bool,
    transform_matrix: Option<[f32; 16]>,
    dirty_params: bool,
    is_spot: bool,
    intensity: Option<f32>,
    color: Option<[f32; 3]>,
    cone_angle: Option<f32>,
    cone_softness: Option<f32>,
}

struct UsdSceneExtractorTask {
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
    usd_data_extractor: Option<UsdDataExtractor>,
    scene: Arc<Mutex<Scene>>,
    meshes: HashMap<String, UsdSceneMesh>,
    sphere_lights: HashMap<String, UsdSceneSphereLight>,
    distant_lights: HashMap<String, UsdSceneDistantLight>,
}
impl UsdSceneExtractorTask {
    pub fn run(
        device: Arc<wgpu::Device>,
        queue: Arc<wgpu::Queue>,
        scene: Arc<Mutex<Scene>>,
        open_usd_receiver: Receiver<String>,
        time_code_receiver: Receiver<i64>,
        stop_receiver: Receiver<()>,
    ) -> std::thread::JoinHandle<()> {
        std::thread::spawn(move || {
            let mut task = Self {
                device,
                queue,
                usd_data_extractor: None,
                scene,
                meshes: HashMap::new(),
                sphere_lights: HashMap::new(),
                distant_lights: HashMap::new(),
            };

            loop {
                let mut filename = None;
                while let Ok(file) = open_usd_receiver.try_recv() {
                    filename = Some(file);
                }
                if let Some(filename) = filename {
                    task.load_usd(&filename);
                }

                let mut time_code = None;
                while let Ok(tc) = time_code_receiver.try_recv() {
                    time_code = Some(tc);
                }
                if let Some(time_code) = time_code {
                    task.set_time_code(time_code);
                }

                if stop_receiver.try_recv().is_ok() {
                    break;
                }
            }
        })
    }

    fn load_usd(&mut self, filename: &str) {
        let mut scene = self.scene.lock().unwrap();
        self.usd_data_extractor = UsdDataExtractor::new(filename)
            .inspect_err(|_| eprintln!("Failed to open USD file: {filename}"))
            .ok();
        *scene = Scene {
            range: None,
            meshes: HashMap::new(),
            distant_lights: HashMap::new(),
            sphere_lights: HashMap::new(),
        };
    }

    fn set_time_code(&mut self, time_code: i64) {
        let Some(usd_data_extractor) = &mut self.usd_data_extractor else {
            return;
        };

        let diff = usd_data_extractor.extract(time_code as f64);
        for data in diff {
            match data {
                BridgeData::TimeCodeRange(start, end) => {
                    let mut scene = self.scene.lock().unwrap();
                    scene.range = Some(TimeCodeRange {
                        start: start as i64,
                        end: end as i64,
                    });
                }
                BridgeData::TransformMatrix(path, matrix) => {
                    if let Some(mesh) = self.meshes.get_mut(&path.0) {
                        mesh.transform_matrix = Some(matrix);
                        mesh.dirty_transform = true;
                    }
                    if let Some(light) = self.distant_lights.get_mut(&path.0) {
                        light.transform_matrix = Some(matrix);
                        light.dirty_transform = true;
                    }
                    if let Some(light) = self.sphere_lights.get_mut(&path.0) {
                        light.transform_matrix = Some(matrix);
                        light.dirty_transform = true;
                    }
                }
                BridgeData::CreateMesh(path) => {
                    self.meshes.insert(
                        path.0,
                        UsdSceneMesh {
                            dirty_transform: true,
                            transform_matrix: None,
                            dirty_mesh: true,
                            left_handed: false,
                            points_data: None,
                            points_interpolation: None,
                            normals_data: None,
                            normals_interpolation: None,
                            uvs_data: None,
                            uvs_interpolation: None,
                            face_vertex_indices: None,
                            face_vertex_counts: None,
                        },
                    );
                }
                BridgeData::MeshData(path, data) => {
                    if let Some(mesh) = self.meshes.get_mut(&path.0) {
                        mesh.left_handed = data.left_handed;
                        mesh.points_data = Some(data.points_data);
                        mesh.points_interpolation = Some(data.points_interpolation);
                        mesh.normals_data = data.normals_data;
                        mesh.normals_interpolation = data.normals_interpolation;
                        mesh.uvs_data = data.uvs_data;
                        mesh.uvs_interpolation = data.uvs_interpolation;
                        mesh.face_vertex_indices = Some(data.face_vertex_indices);
                        mesh.face_vertex_counts = Some(data.face_vertex_counts);
                        mesh.dirty_mesh = true;
                    }
                }
                BridgeData::DestroyMesh(path) => {
                    self.meshes.remove(&path.0);
                }
                BridgeData::CreateDistantLight(path) => {
                    self.distant_lights.insert(
                        path.0,
                        UsdSceneDistantLight {
                            dirty_transform: true,
                            transform_matrix: None,
                            dirty_params: true,
                            intensity: None,
                            color: None,
                            angle: None,
                        },
                    );
                }
                BridgeData::DistantLightData(path, data) => {
                    if let Some(light) = self.distant_lights.get_mut(&path.0) {
                        light.intensity = Some(data.intensity);
                        light.color = Some(data.color);
                        light.dirty_params = true;
                        light.angle = data.angle;
                    }
                }
                BridgeData::DestroyDistantLight(path) => {
                    self.distant_lights.remove(&path.0);
                }
                BridgeData::CreateSphereLight(path) => {
                    self.sphere_lights.insert(
                        path.0,
                        UsdSceneSphereLight {
                            dirty_transform: true,
                            transform_matrix: None,
                            dirty_params: true,
                            is_spot: false,
                            intensity: None,
                            color: None,
                            cone_angle: None,
                            cone_softness: None,
                        },
                    );
                }
                BridgeData::SphereLightData(path, data) => {
                    if let Some(light) = self.sphere_lights.get_mut(&path.0) {
                        light.intensity = Some(data.intensity);
                        light.color = Some(data.color);
                        light.is_spot = data.cone_angle.is_some();
                        light.cone_angle = data.cone_angle;
                        light.cone_softness = data.cone_softness;
                        light.dirty_params = true;
                    }
                }
                BridgeData::DestroySphereLight(path) => {
                    self.sphere_lights.remove(&path.0);
                }
                _ => (),
            }
        }

        self.sync_scene();
    }

    fn sync_scene(&mut self) {
        let mut scene = self.scene.lock().unwrap();
        self.sync_light(&mut scene);
        self.sync_mesh(&mut scene);
    }

    fn sync_light(&self, scene: &mut Scene) {
        // remove deleted lights
        scene
            .distant_lights
            .retain(|path, _| self.distant_lights.contains_key(path));
        scene
            .sphere_lights
            .retain(|path, _| self.sphere_lights.contains_key(path));

        // create new lights
        for (path, _) in &self.distant_lights {
            if !scene.distant_lights.contains_key(path) {
                scene.distant_lights.insert(
                    path.to_owned(),
                    DistantLight {
                        direction: Vec3::Z,
                        intensity: 0.0,
                        color: Vec3::ZERO,
                        angle: 0.0,
                    },
                );
            }
        }
        for (path, _) in &self.sphere_lights {
            if !scene.sphere_lights.contains_key(path) {
                scene.sphere_lights.insert(
                    path.to_owned(),
                    SphereLight {
                        is_spot: false,
                        position: Vec3::ZERO,
                        intensity: 0.0,
                        color: Vec3::ZERO,
                        direction: None,
                        cone_angle: None,
                        cone_softness: None,
                    },
                );
            }
        }

        // update lights
        for (path, light) in &self.distant_lights {
            if light.dirty_transform {
                let scene_light = scene.distant_lights.get_mut(path).unwrap();
                let transform = light
                    .transform_matrix
                    .map(|m| glam::Mat4::from_cols_array(&m))
                    .unwrap_or(glam::Mat4::IDENTITY);
                let direction = transform.transform_vector3(Vec3::Z).normalize();
                scene_light.direction = direction;
            }

            if light.dirty_params {
                let scene_light = scene.distant_lights.get_mut(path).unwrap();
                scene_light.intensity = light.intensity.unwrap_or(0.0);
                scene_light.color = light.color.map_or(Vec3::ZERO, |c| Vec3::from_array(c));
                scene_light.angle = light.angle.unwrap_or(0.0);
            }
        }
        for (path, light) in &self.sphere_lights {
            if light.dirty_transform {
                let scene_light = scene.sphere_lights.get_mut(path).unwrap();
                let transform = light
                    .transform_matrix
                    .map(|m| glam::Mat4::from_cols_array(&m))
                    .unwrap_or(glam::Mat4::IDENTITY);
                let position = transform.transform_point3(Vec3::ZERO);
                let direction = transform.transform_vector3(Vec3::Z).normalize();
                scene_light.position = position;
                scene_light.direction = Some(direction);
            }

            if light.dirty_params {
                let scene_light = scene.sphere_lights.get_mut(path).unwrap();
                scene_light.intensity = light.intensity.unwrap_or(0.0);
                scene_light.color = light.color.map_or(Vec3::ZERO, |c| Vec3::from_array(c));
                scene_light.is_spot = light.is_spot;
                scene_light.cone_angle = light.cone_angle;
                scene_light.cone_softness = light.cone_softness;
            }
        }
    }

    fn sync_mesh(&self, scene: &mut Scene) {
        // remove deleted meshes
        scene
            .meshes
            .retain(|path, _| self.meshes.contains_key(path));

        // create new meshes
        for (path, _) in &self.meshes {
            if !scene.meshes.contains_key(path) {
                let model_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some(&format!("{} Model Buffer", path)),
                    size: std::mem::size_of::<Model>() as u64,
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                });
                scene.meshes.insert(
                    path.to_owned(),
                    Mesh {
                        vertex_count: 0,
                        vertex_buffer: None,
                        model_buffer,
                    },
                );
            }
        }

        // update meshes
        for (path, mesh) in &self.meshes {
            if mesh.dirty_transform {
                let scene_mesh = scene.meshes.get_mut(path).unwrap();
                self.queue.write_buffer(
                    &scene_mesh.model_buffer,
                    0,
                    bytemuck::cast_slice(&[Model {
                        model: mesh
                            .transform_matrix
                            .map(|m| glam::Mat4::from_cols_array(&m))
                            .unwrap_or(glam::Mat4::IDENTITY),
                    }]),
                );
            }

            if mesh.dirty_mesh {
                let (face_varying_triangulate_indices, vertex_triangulate_indices) = {
                    let mut face_varying_triangulate_indices = Vec::new();
                    let mut vertex_triangulate_indices = Vec::new();
                    let (Some(face_vertex_indices), Some(face_vertex_counts)) =
                        (&mesh.face_vertex_indices, &mesh.face_vertex_counts)
                    else {
                        continue;
                    };
                    let mut index_offset = 0;
                    for face_vertex_count in face_vertex_counts {
                        let face_vertex_count = *face_vertex_count as usize;
                        for i in 2..face_vertex_count {
                            vertex_triangulate_indices
                                .push(face_vertex_indices[index_offset] as usize);
                            vertex_triangulate_indices
                                .push(face_vertex_indices[index_offset + i - 1] as usize);
                            vertex_triangulate_indices
                                .push(face_vertex_indices[index_offset + i] as usize);
                            face_varying_triangulate_indices.push(index_offset);
                            face_varying_triangulate_indices.push(index_offset + i - 1);
                            face_varying_triangulate_indices.push(index_offset + i);
                        }
                        index_offset += face_vertex_count;
                    }
                    (face_varying_triangulate_indices, vertex_triangulate_indices)
                };

                let vertex_points = {
                    let (Some(points), Some(points_interpolation)) =
                        (&mesh.points_data, &mesh.points_interpolation)
                    else {
                        continue;
                    };
                    match points_interpolation {
                        Interpolation::FaceVarying => {
                            let points = bytemuck::cast_slice::<f32, Vec3>(&points);
                            let mut data =
                                Vec::with_capacity(face_varying_triangulate_indices.len());
                            for &index in &face_varying_triangulate_indices {
                                data.push(points[index]);
                            }
                            data
                        }
                        Interpolation::Vertex => {
                            let points = bytemuck::cast_slice::<f32, Vec3>(&points);
                            let mut data = Vec::with_capacity(vertex_triangulate_indices.len());
                            for &index in &vertex_triangulate_indices {
                                data.push(points[index]);
                            }
                            data
                        }
                        _ => continue,
                    }
                };

                let vertex_normals = {
                    if let (Some(normals), Some(normals_interpolation)) =
                        (&mesh.normals_data, &mesh.normals_interpolation)
                    {
                        match normals_interpolation {
                            Interpolation::FaceVarying => {
                                let normals = bytemuck::cast_slice::<f32, Vec3>(&normals);
                                let mut data =
                                    Vec::with_capacity(face_varying_triangulate_indices.len());
                                for &index in &face_varying_triangulate_indices {
                                    data.push(normals[index]);
                                }
                                data
                            }
                            Interpolation::Vertex => {
                                let normals = bytemuck::cast_slice::<f32, Vec3>(&normals);
                                let mut data = Vec::with_capacity(vertex_triangulate_indices.len());
                                for &index in &vertex_triangulate_indices {
                                    data.push(normals[index]);
                                }
                                data
                            }
                            _ => continue,
                        }
                    } else {
                        // Calculate normals
                        let Some(points) = &mesh.points_data else {
                            continue;
                        };
                        let points = bytemuck::cast_slice::<f32, Vec3>(points);
                        let mut normals_point = vec![vec![]; points.len()];
                        let indices = match mesh.points_interpolation {
                            Some(Interpolation::FaceVarying) => &face_varying_triangulate_indices,
                            Some(Interpolation::Vertex) => &vertex_triangulate_indices,
                            _ => continue,
                        };
                        for face_indices in indices.chunks(3) {
                            let p0 = points[face_indices[0]];
                            let p1 = points[face_indices[1]];
                            let p2 = points[face_indices[2]];
                            let mut normal = (p1 - p0).cross(p2 - p0).normalize();
                            if mesh.left_handed {
                                normal = -normal;
                            }
                            for &index in face_indices {
                                normals_point[index].push(normal);
                            }
                        }
                        let mut mean_normals = Vec::with_capacity(points.len());
                        for normals in normals_point {
                            let mut mean_normal = Vec3::ZERO;
                            for normal in normals {
                                mean_normal += normal;
                            }
                            mean_normals.push(mean_normal.normalize());
                        }
                        let mut vertex_normals = Vec::with_capacity(indices.len());
                        for &index in indices {
                            vertex_normals.push(mean_normals[index]);
                        }
                        vertex_normals
                    }
                };

                let vertex_uvs = {
                    if let (Some(uvs), Some(uvs_interpolation)) =
                        (&mesh.uvs_data, &mesh.uvs_interpolation)
                    {
                        match uvs_interpolation {
                            Interpolation::FaceVarying => {
                                let uvs = bytemuck::cast_slice::<f32, Vec2>(&uvs);
                                let mut data =
                                    Vec::with_capacity(face_varying_triangulate_indices.len());
                                for &index in &face_varying_triangulate_indices {
                                    data.push(uvs[index]);
                                }
                                Some(data)
                            }
                            Interpolation::Vertex => {
                                let uvs = bytemuck::cast_slice::<f32, Vec2>(&uvs);
                                let mut data = Vec::with_capacity(vertex_triangulate_indices.len());
                                for &index in &vertex_triangulate_indices {
                                    data.push(uvs[index]);
                                }
                                Some(data)
                            }
                            _ => continue,
                        }
                    } else {
                        None
                    }
                };

                let vertex_count = face_varying_triangulate_indices.len();
                let mut vertex_data = Vec::with_capacity(vertex_count);
                if mesh.left_handed {
                    for face in (0..vertex_count).collect::<Vec<_>>().chunks(3) {
                        for i in face.iter().rev() {
                            vertex_data.push(Vertex {
                                position: vertex_points[*i],
                                normal: vertex_normals[*i],
                                uv: vertex_uvs.as_ref().map_or(Vec2::ZERO, |uvs| uvs[*i]),
                            });
                        }
                    }
                } else {
                    for i in 0..vertex_count {
                        vertex_data.push(Vertex {
                            position: vertex_points[i],
                            normal: vertex_normals[i],
                            uv: vertex_uvs.as_ref().map_or(Vec2::ZERO, |uvs| uvs[i]),
                        });
                    }
                }

                let prev_vertex_count = scene.meshes.get(path).map_or(0, |m| m.vertex_count);

                let scene_mesh = scene.meshes.get_mut(path).unwrap();
                if prev_vertex_count != vertex_data.len() as u32
                    || scene_mesh.vertex_buffer.is_none()
                {
                    let vertex_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
                        label: Some(&format!("{} Vertex Buffer", path)),
                        size: (vertex_data.len() * std::mem::size_of::<Vertex>()) as u64,
                        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                        mapped_at_creation: false,
                    });
                    self.queue
                        .write_buffer(&vertex_buffer, 0, bytemuck::cast_slice(&vertex_data));
                    scene_mesh.vertex_buffer = Some(vertex_buffer);
                    scene_mesh.vertex_count = vertex_data.len() as u32;
                } else {
                    let vertex_buffer = scene_mesh.vertex_buffer.as_ref().unwrap();
                    self.queue
                        .write_buffer(vertex_buffer, 0, bytemuck::cast_slice(&vertex_data));
                }
            }
        }
    }
}

pub struct SceneLoader {
    scene: Arc<Mutex<Scene>>,
    open_usd_sender: Sender<String>,
    time_code_sender: Sender<i64>,
    stop_sender: Sender<()>,
    join_handle: Option<std::thread::JoinHandle<()>>,
}
impl SceneLoader {
    pub fn new(device: Arc<wgpu::Device>, queue: Arc<wgpu::Queue>) -> Self {
        let scene = Arc::new(Mutex::new(Scene {
            range: None,
            meshes: HashMap::new(),
            sphere_lights: HashMap::new(),
            distant_lights: HashMap::new(),
        }));
        let (time_code_sender, time_code_receiver) = channel();
        let (open_usd_sender, open_usd_receiver) = channel();
        let (stop_sender, stop_receiver) = channel();

        let join_handle = UsdSceneExtractorTask::run(
            device,
            queue,
            Arc::clone(&scene),
            open_usd_receiver,
            time_code_receiver,
            stop_receiver,
        );

        Self {
            scene,
            open_usd_sender,
            time_code_sender,
            stop_sender,
            join_handle: Some(join_handle),
        }
    }

    pub fn load_usd(&self, filename: &str) {
        self.open_usd_sender.send(filename.to_string()).unwrap();
    }

    pub fn set_time_code(&self, time_code: i64) {
        self.time_code_sender.send(time_code).unwrap();
    }

    pub fn read_scene(&self, f: impl FnOnce(&Scene)) {
        let scene = self.scene.lock().unwrap();
        f(&scene);
    }
}
impl Drop for SceneLoader {
    fn drop(&mut self) {
        self.stop_sender.send(()).unwrap();
        let join_handle = self.join_handle.take().unwrap();
        join_handle.join().unwrap();
    }
}
