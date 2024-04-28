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
pub struct RenderSettings {
    pub settings_paths: Vec<String>,
    pub product_paths: Vec<String>,
    pub current_settings_path: Option<String>,
    pub next_settings_path: Option<Option<String>>,
    pub current_product_path: Option<String>,
    pub next_product_path: Option<Option<String>>,
}

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

enum UsdSceneExtractorMessage {
    LoadUsd(String),
    SetTimeCode(i64),
    SyncRenderSettings,
    Stop,
}

struct SyncItems {
    scene: Scene,
    render_settings: RenderSettings,
}

struct UsdSceneExtractorTask {
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,

    usd_data_extractor: Option<UsdDataExtractor>,

    sync_items: Arc<Mutex<SyncItems>>,

    meshes: HashMap<String, UsdSceneMesh>,
    sphere_lights: HashMap<String, UsdSceneSphereLight>,
    distant_lights: HashMap<String, UsdSceneDistantLight>,
    cameras: HashMap<String, UsdCamera>,
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
                meshes: HashMap::new(),
                sphere_lights: HashMap::new(),
                distant_lights: HashMap::new(),
                cameras: HashMap::new(),
            };

            loop {
                let mut filename = None;
                let mut time_code = None;
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
                        UsdSceneExtractorMessage::SyncRenderSettings => {
                            sync_flag = true;
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

                if sync_flag {
                    task.sync_scene();
                }
            }
        })
    }

    // 裏でusd読み込みのために走っているスレッドで、USDファイルのロードボタンが押されていたら呼び出されるメソッド。
    // 新しくUsdDataExtractorを作成し、syncしているシーン情報などを初期化する。
    fn load_usd(&mut self, filename: &str) {
        let mut sync_items = self.sync_items.lock().unwrap();
        self.usd_data_extractor = UsdDataExtractor::new(filename)
            .inspect_err(|_| eprintln!("Failed to open USD file: {filename}"))
            .ok();
        sync_items.scene = Scene {
            range: None,
            meshes: HashMap::new(),
            distant_lights: HashMap::new(),
            sphere_lights: HashMap::new(),
            camera: Camera::new(),
        };
        sync_items.render_settings = RenderSettings {
            settings_paths: Vec::new(),
            product_paths: Vec::new(),
            current_settings_path: None,
            next_settings_path: None,
            current_product_path: None,
            next_product_path: None,
        };
    }

    // 裏でusd読み込みのために走っているスレッドでtime_codeが変更された際に呼び出されるメソッド。
    // UsdDataExtractorからtime_codeに対応するデータを取得し、シーンとしてアクセスできるようにする。
    fn set_time_code(&mut self, time_code: i64) {
        let Some(usd_data_extractor) = &mut self.usd_data_extractor else {
            return;
        };

        let diff = usd_data_extractor.extract(time_code as f64);
        for data in diff {
            match data {
                BridgeData::TimeCodeRange(start, end) => {
                    let mut sync_items = self.sync_items.lock().unwrap();
                    sync_items.scene.range = Some(TimeCodeRange {
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
                _ => (),
            }
        }
    }

    // レンダリングに必要なシーン情報のsyncを行う。
    fn sync_scene(&mut self) {
        let sync_items = Arc::clone(&self.sync_items);
        let mut sync_items = sync_items.lock().unwrap();
        self.sync_render_settings(&mut sync_items.render_settings);
        self.sync_light(&mut sync_items.scene);
        self.sync_mesh(&mut sync_items.scene);
        self.sync_camera(&mut sync_items.scene);
    }

    // シーン情報をsyncするさいに現在アクティブなRenderSettingsやRenderProductのパスを同期する。
    // これによって設定されたアクティブなRenderSettingsやRenderProductのパスを、
    // レンダリングのカメラを決めたりする際に使う。
    //
    // 外部からsetやclearされた情報がrender_settingsのnext_XXXで手に入るので、
    // その値をUsdDataExtractorにsetやclearして
    // UsdDataExtractorのactiveなRenderSettingsやRenderProductsのパスを同期する。
    // 同期に成功したら、render_settingsのcurrent_XXXにnext_XXXをコピーして、next_XXXをNoneにする。
    //
    // また、それとは別に設定するパスの候補になるRenderSettingsとRenderProductのpathsを取得して
    // render_settingsに設定する。
    fn sync_render_settings(&mut self, render_settings: &mut RenderSettings) {
        // usd_data_extractorがNoneの場合はロード前なので何もしない
        let Some(usd_data_extractor) = &mut self.usd_data_extractor else {
            return;
        };

        // Stage中にある全UsdRenderSettingsのパスを取得
        render_settings.settings_paths = usd_data_extractor.get_render_settings_paths();

        // render_settingsのnext_settings_pathがSomeの場合は、
        // そのnext_pathによってUsdDataExtractorのアクティブなRenderSettingsPathをsetしたりClearしたりする。
        if let Some(next_path) = &render_settings.next_settings_path {
            match next_path {
                // next_pathがSomeの場合は、そのパスをアクティブなRenderSettingsのパスとして
                // UsdDataExtractorにsetする。
                // setはUsdRenderSettingsとして存在しないパスをセットした場合に失敗する。
                // 失敗した場合はそのセットは無視する。
                Some(path) => match usd_data_extractor.set_render_settings_path(path) {
                    Ok(()) => {
                        render_settings.current_settings_path = Some(path.to_string());
                    }
                    Err(_) => {}
                },
                // next_pathがNoneの場合には、アクティブなRenderSettingsPathをClearする。
                None => {
                    usd_data_extractor.clear_render_settings_path();
                    render_settings.current_settings_path = None;

                    // Clearした場合はRenderProductのパスもクリアする。
                    render_settings.product_paths = Vec::new();
                    render_settings.current_product_path = None;
                }
            }
            render_settings.next_settings_path = None;
        }

        // アクティブとしてセットされているUsdRenderSettingsにリレーション登録されている
        // 全UsdRenderProductのパスを取得する。
        match usd_data_extractor.get_render_product_paths() {
            Ok(paths) => render_settings.product_paths = paths,
            Err(_) => {
                render_settings.product_paths = Vec::new();
                render_settings.current_product_path = None;
            }
        }

        // render_settingsのnext_product_pathがSomeの場合は、
        // そのnext_pathによってUsdDataExtractorのアクティブなRenderProductPathをsetしたりClearしたりする。
        if let Some(next_path) = &render_settings.next_product_path {
            match next_path {
                // next_pathがSomeの場合は、そのパスをアクティブなRenderProductのパスとして
                // UsdDataExtractorにsetする。
                // setはUsdRenderProductとして存在しないパスをセットした場合に失敗する。
                // 失敗した場合はそのセットは無視する。
                Some(path) => match usd_data_extractor.set_render_product_path(path) {
                    Ok(()) => {
                        render_settings.current_product_path = Some(path.to_string());
                    }
                    Err(_) => {}
                },
                // next_pathがNoneの場合には、アクティブなRenderProductPathをClearする。
                None => {
                    usd_data_extractor.clear_render_product_path();
                    render_settings.current_product_path = None;
                }
            }
            render_settings.next_product_path = None;
        }
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

    fn sync_camera(&mut self, scene: &mut Scene) {
        // usd_data_extractorがNoneの場合はロード前なので何もしない
        let Some(usd_data_extractor) = &mut self.usd_data_extractor else {
            return;
        };

        // アクティブなカメラのパスを取得
        let camera_path = match usd_data_extractor.get_active_camera_path() {
            Ok(path) => Some(path),
            Err(_) => None,
        };

        // アクティブなカメラの情報を取得する
        let camera = match &camera_path {
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
            current_settings_path: None,
            next_settings_path: None,
            current_product_path: None,
            next_product_path: None,
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
        sync_item.render_settings.settings_paths.clone()
    }

    pub fn set_render_settings_path(&self, path: &str) {
        let mut sync_item = self.sync_item.lock().unwrap();
        sync_item.render_settings.next_settings_path = Some(Some(path.to_string()));
        self.message_sender
            .send(UsdSceneExtractorMessage::SyncRenderSettings)
            .unwrap();
    }

    pub fn clear_render_settings_path(&self) {
        let mut sync_item = self.sync_item.lock().unwrap();
        sync_item.render_settings.next_settings_path = Some(None);
        self.message_sender
            .send(UsdSceneExtractorMessage::SyncRenderSettings)
            .unwrap();
    }

    pub fn get_render_product_paths(&self) -> Vec<String> {
        let sync_item = self.sync_item.lock().unwrap();
        sync_item.render_settings.product_paths.clone()
    }

    pub fn set_render_product_path(&self, path: &str) {
        let mut sync_item = self.sync_item.lock().unwrap();
        sync_item.render_settings.next_product_path = Some(Some(path.to_string()));
        self.message_sender
            .send(UsdSceneExtractorMessage::SyncRenderSettings)
            .unwrap();
    }

    pub fn clear_render_product_path(&self) {
        let mut sync_item = self.sync_item.lock().unwrap();
        sync_item.render_settings.next_product_path = Some(None);
        self.message_sender
            .send(UsdSceneExtractorMessage::SyncRenderSettings)
            .unwrap();
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
