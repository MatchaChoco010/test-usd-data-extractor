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
use crate::scene::*;

#[derive(Debug)]
struct RenderSettings {
    settings_paths: Vec<String>,
    product_paths: Vec<String>,
    active_settings_path: Option<String>,
    active_product_path: Option<String>,
}

#[derive(Debug)]
struct SyncItems {
    scene: Scene,
    render_settings: RenderSettings,
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

#[derive(Debug)]
struct UsdCamera {
    transform_matrix: [f32; 16],
    focal_length: f32,
    vertical_aperture: f32,
}

#[derive(Debug)]
struct UsdRenderSettings {
    product_paths: Vec<String>,
}

#[derive(Debug)]
struct UsdRenderProduct {
    camera_path: String,
}

enum UsdSceneExtractorMessage {
    LoadUsd(String),
    SetTimeCode(i64),
    SetActiveRenderSettings(Option<String>),
    SetActiveRenderProduct(Option<String>),
    Stop,
}

struct UsdSceneExtractorTask {
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,

    usd_data_extractor: Option<UsdDataExtractor>,

    sync_items: Arc<Mutex<SyncItems>>,

    active_render_settings_path: Option<String>,
    active_render_product_path: Option<String>,

    meshes: HashMap<String, UsdSceneMesh>,
    sphere_lights: HashMap<String, UsdSceneSphereLight>,
    distant_lights: HashMap<String, UsdSceneDistantLight>,
    cameras: HashMap<String, UsdCamera>,
    render_settings: HashMap<String, UsdRenderSettings>,
    render_products: HashMap<String, UsdRenderProduct>,
}
impl UsdSceneExtractorTask {
    // 裏でusd読み込みのために走っているスレッドのエントリーポイント。
    // スレッドでは無限ループ内で外部からのメッセージを監視し、、
    // それぞれのメッセージが来たらload_usd, set_time_code, stopを呼び出す。
    pub fn run(
        device: Arc<wgpu::Device>,
        queue: Arc<wgpu::Queue>,
        sync_items: Arc<Mutex<SyncItems>>,
        receiver: Receiver<UsdSceneExtractorMessage>,
    ) -> std::thread::JoinHandle<()> {
        std::thread::spawn(move || {
            let mut task = Self {
                device,
                queue,
                usd_data_extractor: None,
                sync_items,
                active_render_settings_path: None,
                active_render_product_path: None,
                meshes: HashMap::new(),
                sphere_lights: HashMap::new(),
                distant_lights: HashMap::new(),
                cameras: HashMap::new(),
                render_settings: HashMap::new(),
                render_products: HashMap::new(),
            };

            loop {
                let mut filename = None;
                let mut time_code = None;
                let mut active_render_settings_path = None;
                let mut active_render_product_path = None;
                let mut sync_flag = false;
                while let Ok(message) = receiver.recv() {
                    match message {
                        UsdSceneExtractorMessage::LoadUsd(file) => {
                            filename = Some(file);
                            break;
                        }
                        UsdSceneExtractorMessage::SetTimeCode(tc) => {
                            time_code = Some(tc);
                            break;
                        }
                        UsdSceneExtractorMessage::SetActiveRenderSettings(path) => {
                            active_render_settings_path = Some(path);
                            break;
                        }
                        UsdSceneExtractorMessage::SetActiveRenderProduct(path) => {
                            active_render_product_path = Some(path);
                            break;
                        }
                        UsdSceneExtractorMessage::Stop => {
                            return;
                        }
                    }
                }

                if let Some(filename) = filename {
                    task.load_usd(&filename);
                }

                if let Some(time_code) = time_code {
                    task.set_time_code(time_code);
                    sync_flag = true;
                }

                if let Some(path) = active_render_settings_path {
                    task.set_active_render_settings_path(path);
                    sync_flag = true;
                }

                if let Some(path) = active_render_product_path {
                    task.set_active_render_product_path(path);
                    sync_flag = true;
                }

                if sync_flag {
                    task.sync_scene();
                }
            }
        })
    }

    // 裏でusd読み込みのために走っているスレッドで、USDファイルのロードボタンが押されていたら呼び出されるメソッド。
    // 新しくUsdDataExtractorを作成し、syncしているシーン情報などを初期化する。
    fn load_usd(&mut self, filename: &str) {
        let mut scene = self.sync_items.lock().unwrap();
        self.usd_data_extractor = UsdDataExtractor::new(filename)
            .inspect_err(|_| eprintln!("Failed to open USD file: {filename}"))
            .ok();
        scene.scene = Scene {
            range: None,
            meshes: HashMap::new(),
            distant_lights: HashMap::new(),
            sphere_lights: HashMap::new(),
            camera: Camera::new(),
        };
        scene.render_settings = RenderSettings {
            settings_paths: Vec::new(),
            product_paths: Vec::new(),
            active_settings_path: None,
            active_product_path: None,
        };
    }

    // 裏でusd読み込みのために走っているスレッドでtime_codeが変更された際に呼び出されるメソッド。
    // UsdDataExtractorからtime_codeに対応するデータを取得し、
    // UsdSceneExtractorのメンバ変数に反映する。
    fn set_time_code(&mut self, time_code: i64) {
        let Some(usd_data_extractor) = &mut self.usd_data_extractor else {
            return;
        };

        let diff = usd_data_extractor.extract(time_code as f64);
        for data in diff {
            match data {
                BridgeData::TimeCodeRange(start, end) => {
                    let mut scene = self.sync_items.lock().unwrap();
                    scene.scene.range = Some(TimeCodeRange {
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
                    if let Some(camera) = self.cameras.get_mut(&path.0) {
                        camera.transform_matrix = matrix;
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
                BridgeData::CreateCamera(path) => {
                    self.cameras.insert(
                        path.0,
                        UsdCamera {
                            transform_matrix: [0.0; 16],
                            focal_length: 16.0,
                            vertical_aperture: 23.8,
                        },
                    );
                }
                BridgeData::CameraData(path, data) => {
                    if let Some(camera) = self.cameras.get_mut(&path.0) {
                        camera.focal_length = data.focal_length;
                        camera.vertical_aperture = data.vertical_aperture;
                    }
                }
                BridgeData::DestroyCamera(path) => {
                    self.cameras.remove(&path.0);
                }
                BridgeData::CreateRenderSettings(path) => {
                    self.render_settings.insert(
                        path.0,
                        UsdRenderSettings {
                            product_paths: Vec::new(),
                        },
                    );
                }
                BridgeData::RenderSettingsData(path, data) => {
                    if let Some(render_settings) = self.render_settings.get_mut(&path.0) {
                        render_settings.product_paths = data.render_product_paths;
                    }
                }
                BridgeData::DestroyRenderSettings(path) => {
                    self.render_settings.remove(&path.0);
                }
                BridgeData::CreateRenderProduct(path) => {
                    self.render_products.insert(
                        path.0,
                        UsdRenderProduct {
                            camera_path: String::new(),
                        },
                    );
                }
                BridgeData::RenderProductData(path, data) => {
                    if let Some(render_product) = self.render_products.get_mut(&path.0) {
                        render_product.camera_path = data.camera_path;
                    }
                }
                BridgeData::DestroyRenderProduct(path) => {
                    self.render_products.remove(&path.0);
                }
                _ => (),
            }
        }
    }

    // 裏でusd読み込みのために走っているスレッドでSetActiveRenderSettingsが呼ばれた際に呼び出されるメソッド。
    // 渡されたpathがステージに存在しているかを確認している。
    // 存在しない場合はactiveなRenderSettingsとRenderProductの設定をクリアする。

    fn set_active_render_settings_path(&mut self, path: Option<String>) {
        match path {
            Some(path) => {
                let has_path = self.render_settings.contains_key(&path);
                if has_path {
                    self.active_render_settings_path = Some(path.clone());
                    self.active_render_product_path = None;
                } else {
                    self.active_render_settings_path = None;
                    self.active_render_product_path = None;
                }
            }
            None => {
                self.active_render_settings_path = None;
                self.active_render_product_path = None;
            }
        }
    }

    // 裏でusd読み込みのために走っているスレッドでSetActiveRenderProductが呼ばれた際に呼び出されるメソッド。
    // 渡されたpathが現在のアクティブなRenderSettingsに存在しているかを確認している。
    // 存在しない場合はactiveなRenderProductの設定をクリアする。
    fn set_active_render_product_path(&mut self, path: Option<String>) {
        match path {
            Some(path) => {
                let Some(render_settings) = self.render_settings.get(&path) else {
                    self.active_render_product_path = None;
                    return;
                };
                let has_path = render_settings.product_paths.contains(&path);
                if has_path {
                    self.active_render_product_path = Some(path.clone());
                } else {
                    self.active_render_product_path = None;
                }
            }
            None => {
                self.active_render_product_path = None;
            }
        }
    }

    // レンダリングに必要なシーン情報のsyncを行う。
    // UsdSceneExtractorのメンバ変数にあるシーン情報を、sceneのシーン情報にコピーしていく。
    fn sync_scene(&mut self) {
        let scene = Arc::clone(&self.sync_items);
        let mut scene = scene.lock().unwrap();
        self.sync_render_settings(&mut scene.render_settings);
        self.sync_light(&mut scene.scene);
        self.sync_mesh(&mut scene.scene);
        self.sync_camera(&mut scene.scene);
    }

    // 設定するパスの候補になるRenderSettingsとRenderProductのpathsを取得して
    // sync_items.render_settingsに設定する。
    // 現在アクティブなRenderSettingsとRenderProductのパスも設定する。
    fn sync_render_settings(&mut self, sync_settings: &mut RenderSettings) {
        // Stage中にある全UsdRenderSettingsのパスを取得
        sync_settings.settings_paths = self.render_settings.keys().cloned().collect();

        // アクティブなRenderSettingsパスを取得する
        sync_settings.active_settings_path = self.active_render_settings_path.clone();

        // アクティブなRenderSettingsがある場合はそのRenderProductのパスを取得する
        match &self.active_render_settings_path {
            Some(active_settings_path) => match self.render_settings.get(active_settings_path) {
                Some(render_settings) => {
                    sync_settings.product_paths = render_settings.product_paths.clone();
                }
                None => {
                    sync_settings.product_paths = Vec::new();
                }
            },
            None => {
                sync_settings.product_paths = Vec::new();
            }
        }

        // アクティブなRenderProductのパスを取得する
        sync_settings.active_product_path = self.active_render_product_path.clone();
    }

    // シーンのライトの情報を同期する。
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

    // シーンのメッシュの情報を同期する。
    // トポロジのInterpolationが頂点属性によって違うので注意。
    // 今回はInterpolationはVertexとFaceVaryingのみをサポートする。
    // 各メッシュは三角形化も行う。
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

    // アクティブなRenderSettingsのRenderProductからカメラのパスを取得して設定する。
    fn sync_camera(&mut self, scene: &mut Scene) {
        // アクティブなRenderProductのパスを取得する
        let camera_path = match &self.active_render_product_path {
            Some(active_render_product_path) => {
                match self.render_products.get(active_render_product_path) {
                    Some(render_product) => Some(&render_product.camera_path),
                    None => None,
                }
            }
            None => None,
        };

        // アクティブなカメラの情報を取得する
        let camera = match camera_path {
            Some(path) => self.cameras.get(path),
            None => None,
        };

        // カメラの情報をシーンに反映する
        match camera {
            Some(camera) => {
                let transform = glam::Mat4::from_cols_array(&camera.transform_matrix);
                let position = transform.transform_point3(Vec3::ZERO);
                let direction = transform.transform_vector3(Vec3::NEG_Z).normalize();
                scene.camera.view_matrix =
                    glam::Mat4::look_at_rh(position, position + direction, Vec3::Y);
                let fovy = 2.0 * (camera.vertical_aperture / 2.0 / camera.focal_length).atan();
                scene.camera.fovy = fovy;
            }
            None => {
                scene.camera.view_matrix = glam::Mat4::look_at_rh(
                    Vec3::new(0.0, 1.8, 5.0),
                    Vec3::new(0.0, 0.8, 0.0),
                    Vec3::Y,
                );
                scene.camera.fovy = 60.0_f32.to_radians();
            }
        }
    }
}

pub struct SceneLoader {
    sync_item: Arc<Mutex<SyncItems>>,
    message_sender: Sender<UsdSceneExtractorMessage>,
    join_handle: Option<std::thread::JoinHandle<()>>,
}
impl SceneLoader {
    pub fn new(device: Arc<wgpu::Device>, queue: Arc<wgpu::Queue>) -> Self {
        let scene = Scene {
            range: None,
            meshes: HashMap::new(),
            sphere_lights: HashMap::new(),
            distant_lights: HashMap::new(),
            camera: Camera::new(),
        };
        let render_settings = RenderSettings {
            settings_paths: Vec::new(),
            product_paths: Vec::new(),
            active_settings_path: None,
            active_product_path: None,
        };
        let sync_item = Arc::new(Mutex::new(SyncItems {
            scene,
            render_settings,
        }));
        let (message_sender, message_receiver) = channel();

        let join_handle =
            UsdSceneExtractorTask::run(device, queue, Arc::clone(&sync_item), message_receiver);

        Self {
            sync_item,
            message_sender,
            join_handle: Some(join_handle),
        }
    }

    pub fn load_usd(&self, filename: &str) {
        self.message_sender
            .send(UsdSceneExtractorMessage::LoadUsd(filename.to_string()))
            .unwrap();
    }

    pub fn set_time_code(&self, time_code: i64) {
        self.message_sender
            .send(UsdSceneExtractorMessage::SetTimeCode(time_code))
            .unwrap();
    }

    pub fn read_scene(&self, f: impl FnOnce(&Scene)) {
        let sync_item = self.sync_item.lock().unwrap();
        f(&sync_item.scene);
    }

    pub fn get_render_settings_paths(&self) -> Vec<String> {
        let sync_item = self.sync_item.lock().unwrap();
        sync_item
            .render_settings
            .settings_paths
            .iter()
            .cloned()
            .filter(|s| !s.is_empty())
            .collect()
    }

    pub fn set_active_render_settings_path(&self, path: Option<&str>) {
        self.message_sender
            .send(UsdSceneExtractorMessage::SetActiveRenderSettings(
                path.map(|s| s.to_string()),
            ))
            .unwrap();
    }

    pub fn get_active_render_settings_path(&self) -> Option<String> {
        let sync_item = self.sync_item.lock().unwrap();
        sync_item.render_settings.active_settings_path.clone()
    }

    pub fn get_render_product_paths(&self) -> Vec<String> {
        let sync_item = self.sync_item.lock().unwrap();
        sync_item.render_settings.product_paths.clone()
    }

    pub fn set_active_render_product_path(&self, path: Option<&str>) {
        self.message_sender
            .send(UsdSceneExtractorMessage::SetActiveRenderProduct(
                path.map(|s| s.to_string()),
            ))
            .unwrap();
    }

    pub fn get_active_render_product_path(&self) -> Option<String> {
        let sync_item = self.sync_item.lock().unwrap();
        sync_item.render_settings.active_product_path.clone()
    }
}
impl Drop for SceneLoader {
    fn drop(&mut self) {
        self.message_sender
            .send(UsdSceneExtractorMessage::Stop)
            .unwrap();
        let join_handle = self.join_handle.take().unwrap();
        join_handle.join().unwrap();
    }
}
