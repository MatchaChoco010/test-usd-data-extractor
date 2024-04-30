use std::collections::HashMap;

#[cxx::bridge]
pub mod ffi {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    pub enum Interpolation {
        Constant,
        Uniform,
        Varying,
        Vertex,
        FaceVarying,
        Instance,
    }

    extern "Rust" {
        // type MeshData;
        // fn new_mesh_data() -> Box<MeshData>;
        // fn set_left_handed(self: &mut MeshData, left_handed: bool);
        // fn set_points(self: &mut MeshData, data: &[f32], interpolation: u8);
        // fn set_normals(self: &mut MeshData, data: &[f32], interpolation: u8);
        // fn set_uvs(self: &mut MeshData, data: &[f32], interpolation: u8);
        // fn set_face_vertex_indices(self: &mut MeshData, data: &[u32]);
        // fn set_face_vertex_counts(self: &mut MeshData, data: &[u32]);

        // type DistantLightData;
        // fn new_distant_light_data() -> Box<DistantLightData>;
        // fn set_intensity(self: &mut DistantLightData, intensity: f32);
        // fn set_color(self: &mut DistantLightData, r: f32, g: f32, b: f32);
        // fn set_angle(self: &mut DistantLightData, angle: f32);

        // type SphereLightData;
        // fn new_sphere_light_data() -> Box<SphereLightData>;
        // fn set_intensity(self: &mut SphereLightData, intensity: f32);
        // fn set_color(self: &mut SphereLightData, r: f32, g: f32, b: f32);
        // fn set_cone_angle(self: &mut SphereLightData, angle: f32);
        // fn set_cone_softness(self: &mut SphereLightData, softness: f32);

        // type CameraData;
        // fn new_camera_data() -> Box<CameraData>;
        // fn set_focal_length(self: &mut CameraData, focal_length: f32);
        // fn set_vertical_aperture(self: &mut CameraData, vertical_aperture: f32);

        // type RenderSettingsData;
        // fn new_render_settings_data() -> Box<RenderSettingsData>;
        // fn set_render_product_paths(self: &mut RenderSettingsData, paths: &[String]);

        // type RenderProductData;
        // fn new_render_product_data() -> Box<RenderProductData>;
        // fn set_camera_path(self: &mut RenderProductData, path: String);

        // type BridgeSender;
        // fn message(self: &BridgeSender, s: String);
        // fn time_code_range(self: &BridgeSender, start: f64, end: f64);
        // fn transform_matrix(self: &BridgeSender, path: String, matrix: &[f64]);
        // fn create_mesh(self: &BridgeSender, path: String);
        // fn mesh_data(self: &BridgeSender, path: String, data: Box<MeshData>);
        // fn destroy_mesh(self: &BridgeSender, path: String);
        // fn create_distant_light(self: &BridgeSender, path: String);
        // fn distant_light_data(self: &BridgeSender, path: String, data: Box<DistantLightData>);
        // fn destroy_distant_light(self: &BridgeSender, path: String);
        // fn create_sphere_light(self: &BridgeSender, path: String);
        // fn sphere_light_data(self: &BridgeSender, path: String, data: Box<SphereLightData>);
        // fn destroy_sphere_light(self: &BridgeSender, path: String);
        // fn create_camera(self: &BridgeSender, path: String);
        // fn camera_data(self: &BridgeSender, path: String, data: Box<CameraData>);
        // fn destroy_camera(self: &BridgeSender, path: String);
        // fn create_render_settings(self: &BridgeSender, path: String);
        // fn render_settings_data(self: &BridgeSender, path: String, data: Box<RenderSettingsData>);
        // fn destroy_render_settings(self: &BridgeSender, path: String);
        // fn create_render_product(self: &BridgeSender, path: String);
        // fn render_product_data(self: &BridgeSender, path: String, data: Box<RenderProductData>);
        // fn destroy_render_product(self: &BridgeSender, path: String);

        type UsdDataDiff;

        // meshが生成されたdiffの記録とそのデータを設定する関数
        fn create_mesh(&mut self, path: String);
        fn create_mesh_transform_matrix(&mut self, path: String, matrix: &[f32]);
        fn create_mesh_left_handed(&mut self, path: String, left_handed: bool);
        fn create_mesh_points(&mut self, path: String, data: &[f32]);
        fn create_mesh_normals(&mut self, path: String, data: &[f32]);
        fn create_mesh_normals_interpolation(&mut self, path: String, interpolation: Interpolation);
        fn create_mesh_uvs(&mut self, path: String, data: &[f32]);
        fn create_mesh_uvs_interpolation(&mut self, path: String, interpolation: Interpolation);
        fn create_mesh_face_vertex_indices(&mut self, path: String, data: &[u32]);
        fn create_mesh_face_vertex_counts(&mut self, path: String, data: &[u32]);
        fn create_mesh_geom_subset(
            &mut self,
            path: String,
            name: String,
            ty_: String,
            indices: &[u32],
        );

        // meshが削除されたdiffを記録する関数
        fn destroy_mesh(&mut self, path: String);

        // meshのtransform matrix情報が編集されたことを記録する関数
        fn diff_mesh_transform_matrix(&mut self, path: String, matrix: &[f32]);

        // meshの頂点データが編集されたdiffの記録とそのデータを設定する関数
        fn diff_mesh_data(&mut self, path: String);
        fn diff_mesh_data_left_handed(&mut self, path: String, left_handed: bool);
        fn diff_mesh_data_points(&mut self, path: String, data: &[f32]);
        fn diff_mesh_data_normals(&mut self, path: String, data: &[f32]);
        fn diff_mesh_data_normals_interpolation(
            &mut self,
            path: String,
            interpolation: Interpolation,
        );
        fn diff_mesh_data_uvs(&mut self, path: String, data: &[f32]);
        fn diff_mesh_data_uvs_interpolation(&mut self, path: String, interpolation: Interpolation);
        fn diff_mesh_data_face_vertex_indices(&mut self, path: String, data: &[u32]);
        fn diff_mesh_data_face_vertex_counts(&mut self, path: String, data: &[u32]);
        fn diff_mesh_data_geom_subset(
            &mut self,
            path: String,
            name: String,
            ty_: String,
            indices: &[u32],
        );

    }
    unsafe extern "C++" {
        include!("usd_data_extractor/cpp/usdDataExtractor.h");

        type BridgeUsdDataExtractor;
        fn new_usd_data_extractor(open_path: &str) -> Result<UniquePtr<BridgeUsdDataExtractor>>;
        fn start_time_code(self: &BridgeUsdDataExtractor) -> f64;
        fn end_time_code(self: &BridgeUsdDataExtractor) -> f64;
        fn extract(
            self: Pin<&mut BridgeUsdDataExtractor>,
            time_code: f64,
            scene_diff: Pin<&mut UsdDataDiff>,
        );
    }
}

pub use ffi::Interpolation;

#[derive(Debug, Hash, PartialEq, Eq)]
pub struct SdfPath(String);
impl Into<String> for SdfPath {
    fn into(self) -> String {
        self.0
    }
}

#[derive(Debug, Default)]
pub struct MeshCreate {
    pub transform_matrix: Option<[f32; 16]>,
    pub left_handed: Option<bool>,
    pub points: Option<Vec<f32>>,
    pub normals: Option<Vec<f32>>,
    pub normals_interpolation: Option<Interpolation>,
    pub uvs: Option<Vec<f32>>,
    pub uvs_interpolation: Option<Interpolation>,
    pub face_vertex_indices: Option<Vec<u32>>,
    pub face_vertex_counts: Option<Vec<u32>>,
    pub geom_subsets: HashMap<String, (String, Vec<u32>)>,
}

#[derive(Debug, Default)]
pub struct MeshDataDiff {
    pub left_handed: Option<bool>,
    pub points: Option<Vec<f32>>,
    pub normals: Option<Vec<f32>>,
    pub normals_interpolation: Option<Interpolation>,
    pub uvs: Option<Vec<f32>>,
    pub uvs_interpolation: Option<Interpolation>,
    pub face_vertex_indices: Option<Vec<u32>>,
    pub face_vertex_counts: Option<Vec<u32>>,
    pub geom_subsets: HashMap<String, (String, Vec<u32>)>,
}

#[derive(Debug, Default)]
pub struct MeshesDiff {
    pub create: HashMap<SdfPath, MeshCreate>,
    pub destroy: Vec<SdfPath>,
    pub diff_transform_matrix: HashMap<SdfPath, [f32; 16]>,
    pub diff_mesh_data: HashMap<SdfPath, MeshDataDiff>,
}

#[derive(Debug, Default)]
pub struct UsdDataDiff {
    pub meshes: MeshesDiff,
}
impl UsdDataDiff {
    fn create_mesh(&mut self, path: String) {
        self.meshes
            .create
            .insert(SdfPath(path), MeshCreate::default());
    }

    fn create_mesh_transform_matrix(&mut self, path: String, matrix: &[f32]) {
        let data = matrix[0..16].try_into().unwrap();
        if let Some(create) = self.meshes.create.get_mut(&SdfPath(path)) {
            create.transform_matrix = Some(data);
        }
    }

    fn create_mesh_left_handed(&mut self, path: String, left_handed: bool) {
        if let Some(create) = self.meshes.create.get_mut(&SdfPath(path)) {
            create.left_handed = Some(left_handed);
        }
    }

    fn create_mesh_points(&mut self, path: String, data: &[f32]) {
        if let Some(create) = self.meshes.create.get_mut(&SdfPath(path)) {
            create.points = Some(data.to_vec());
        }
    }

    fn create_mesh_normals(&mut self, path: String, data: &[f32]) {
        if let Some(create) = self.meshes.create.get_mut(&SdfPath(path)) {
            create.normals = Some(data.to_vec());
        }
    }

    fn create_mesh_normals_interpolation(&mut self, path: String, interpolation: Interpolation) {
        if let Some(create) = self.meshes.create.get_mut(&SdfPath(path)) {
            create.normals_interpolation = Some(interpolation);
        }
    }

    fn create_mesh_uvs(&mut self, path: String, data: &[f32]) {
        if let Some(create) = self.meshes.create.get_mut(&SdfPath(path)) {
            create.uvs = Some(data.to_vec());
        }
    }

    fn create_mesh_uvs_interpolation(&mut self, path: String, interpolation: Interpolation) {
        if let Some(create) = self.meshes.create.get_mut(&SdfPath(path)) {
            create.uvs_interpolation = Some(interpolation);
        }
    }

    fn create_mesh_face_vertex_indices(&mut self, path: String, data: &[u32]) {
        if let Some(create) = self.meshes.create.get_mut(&SdfPath(path)) {
            create.face_vertex_indices = Some(data.to_vec());
        }
    }

    fn create_mesh_face_vertex_counts(&mut self, path: String, data: &[u32]) {
        if let Some(create) = self.meshes.create.get_mut(&SdfPath(path)) {
            create.face_vertex_counts = Some(data.to_vec());
        }
    }

    fn create_mesh_geom_subset(
        &mut self,
        path: String,
        name: String,
        ty_: String,
        indices: &[u32],
    ) {
        if let Some(create) = self.meshes.create.get_mut(&SdfPath(path)) {
            create.geom_subsets.insert(name, (ty_, indices.to_vec()));
        }
    }

    fn destroy_mesh(&mut self, path: String) {
        self.meshes.destroy.push(SdfPath(path));
    }

    fn diff_mesh_transform_matrix(&mut self, path: String, matrix: &[f32]) {
        let data = matrix[0..16].try_into().unwrap();
        self.meshes
            .diff_transform_matrix
            .insert(SdfPath(path), data);
    }

    fn diff_mesh_data(&mut self, path: String) {
        self.meshes
            .diff_mesh_data
            .insert(SdfPath(path), MeshDataDiff::default());
    }

    fn diff_mesh_data_left_handed(&mut self, path: String, left_handed: bool) {
        if let Some(diff) = self.meshes.diff_mesh_data.get_mut(&SdfPath(path)) {
            diff.left_handed = Some(left_handed);
        }
    }

    fn diff_mesh_data_points(&mut self, path: String, data: &[f32]) {
        if let Some(diff) = self.meshes.diff_mesh_data.get_mut(&SdfPath(path)) {
            diff.points = Some(data.to_vec());
        }
    }

    fn diff_mesh_data_normals(&mut self, path: String, data: &[f32]) {
        if let Some(diff) = self.meshes.diff_mesh_data.get_mut(&SdfPath(path)) {
            diff.normals = Some(data.to_vec());
        }
    }

    fn diff_mesh_data_normals_interpolation(&mut self, path: String, interpolation: Interpolation) {
        if let Some(diff) = self.meshes.diff_mesh_data.get_mut(&SdfPath(path)) {
            diff.normals_interpolation = Some(interpolation);
        }
    }

    fn diff_mesh_data_uvs(&mut self, path: String, data: &[f32]) {
        if let Some(diff) = self.meshes.diff_mesh_data.get_mut(&SdfPath(path)) {
            diff.uvs = Some(data.to_vec());
        }
    }

    fn diff_mesh_data_uvs_interpolation(&mut self, path: String, interpolation: Interpolation) {
        if let Some(diff) = self.meshes.diff_mesh_data.get_mut(&SdfPath(path)) {
            diff.uvs_interpolation = Some(interpolation);
        }
    }

    fn diff_mesh_data_face_vertex_indices(&mut self, path: String, data: &[u32]) {
        if let Some(diff) = self.meshes.diff_mesh_data.get_mut(&SdfPath(path)) {
            diff.face_vertex_indices = Some(data.to_vec());
        }
    }

    fn diff_mesh_data_face_vertex_counts(&mut self, path: String, data: &[u32]) {
        if let Some(diff) = self.meshes.diff_mesh_data.get_mut(&SdfPath(path)) {
            diff.face_vertex_counts = Some(data.to_vec());
        }
    }

    fn diff_mesh_data_geom_subset(
        &mut self,
        path: String,
        name: String,
        ty_: String,
        indices: &[u32],
    ) {
        if let Some(diff) = self.meshes.diff_mesh_data.get_mut(&SdfPath(path)) {
            diff.geom_subsets.insert(name, (ty_, indices.to_vec()));
        }
    }
}

// pub fn new_mesh_data() -> Box<MeshData> {
//     Box::new(MeshData {
//         left_handed: false,
//         points_data: None,
//         points_interpolation: None,
//         normals_data: None,
//         normals_interpolation: None,
//         uvs_data: None,
//         uvs_interpolation: None,
//         face_vertex_indices: None,
//         face_vertex_counts: None,
//     })
// }

// pub struct MeshData {
//     pub left_handed: bool,
//     pub points_data: Option<Vec<f32>>,
//     pub points_interpolation: Option<Interpolation>,
//     pub normals_data: Option<Vec<f32>>,
//     pub normals_interpolation: Option<Interpolation>,
//     pub uvs_data: Option<Vec<f32>>,
//     pub uvs_interpolation: Option<Interpolation>,
//     pub face_vertex_indices: Option<Vec<u32>>,
//     pub face_vertex_counts: Option<Vec<u32>>,
// }
// impl MeshData {
//     pub fn set_left_handed(&mut self, left_handed: bool) {
//         self.left_handed = left_handed;
//     }

//     pub fn set_points(&mut self, data: &[f32], interpolation: u8) {
//         self.points_data = Some(data.to_vec());
//         self.points_interpolation = Some(interpolation.into());
//     }

//     pub fn set_normals(&mut self, data: &[f32], interpolation: u8) {
//         self.normals_data = Some(data.to_vec());
//         self.normals_interpolation = Some(interpolation.into());
//     }

//     pub fn set_uvs(&mut self, data: &[f32], interpolation: u8) {
//         self.uvs_data = Some(data.to_vec());
//         self.uvs_interpolation = Some(interpolation.into());
//     }

//     pub fn set_face_vertex_indices(&mut self, data: &[u32]) {
//         self.face_vertex_indices = Some(data.to_vec());
//     }

//     pub fn set_face_vertex_counts(&mut self, data: &[u32]) {
//         self.face_vertex_counts = Some(data.to_vec());
//     }
// }

// pub fn new_distant_light_data() -> Box<DistantLightData> {
//     Box::new(DistantLightData {
//         intensity: 0.0,
//         color: [0.0, 0.0, 0.0],
//         angle: None,
//     })
// }

// pub struct DistantLightData {
//     pub intensity: f32,
//     pub color: [f32; 3],
//     pub angle: Option<f32>,
// }
// impl DistantLightData {
//     pub fn set_intensity(&mut self, intensity: f32) {
//         self.intensity = intensity;
//     }

//     pub fn set_color(&mut self, r: f32, g: f32, b: f32) {
//         self.color = [r, g, b];
//     }

//     pub fn set_angle(&mut self, angle: f32) {
//         self.angle = Some(angle);
//     }
// }

// pub fn new_sphere_light_data() -> Box<SphereLightData> {
//     Box::new(SphereLightData {
//         intensity: 0.0,
//         color: [0.0, 0.0, 0.0],
//         cone_angle: None,
//         cone_softness: None,
//     })
// }

// pub struct SphereLightData {
//     pub intensity: f32,
//     pub color: [f32; 3],
//     pub cone_angle: Option<f32>,
//     pub cone_softness: Option<f32>,
// }
// impl SphereLightData {
//     pub fn set_intensity(&mut self, intensity: f32) {
//         self.intensity = intensity;
//     }

//     pub fn set_color(&mut self, r: f32, g: f32, b: f32) {
//         self.color = [r, g, b];
//     }

//     pub fn set_cone_angle(&mut self, angle: f32) {
//         self.cone_angle = Some(angle);
//     }

//     pub fn set_cone_softness(&mut self, softness: f32) {
//         self.cone_softness = Some(softness);
//     }
// }

// pub fn new_camera_data() -> Box<CameraData> {
//     Box::new(CameraData {
//         focal_length: 16.0,
//         vertical_aperture: 23.8,
//     })
// }

// pub struct CameraData {
//     pub focal_length: f32,
//     pub vertical_aperture: f32,
// }
// impl CameraData {
//     pub fn set_focal_length(&mut self, focal_length: f32) {
//         self.focal_length = focal_length;
//     }

//     pub fn set_vertical_aperture(&mut self, vertical_aperture: f32) {
//         self.vertical_aperture = vertical_aperture;
//     }
// }

// pub fn new_render_settings_data() -> Box<RenderSettingsData> {
//     Box::new(RenderSettingsData {
//         render_product_paths: Vec::new(),
//     })
// }

// pub struct RenderSettingsData {
//     pub render_product_paths: Vec<String>,
// }
// impl RenderSettingsData {
//     pub fn set_render_product_paths(&mut self, paths: &[String]) {
//         self.render_product_paths = paths.to_vec();
//     }
// }

// pub fn new_render_product_data() -> Box<RenderProductData> {
//     Box::new(RenderProductData {
//         camera_path: String::new(),
//     })
// }

// pub struct RenderProductData {
//     pub camera_path: String,
// }
// impl RenderProductData {
//     pub fn set_camera_path(&mut self, path: String) {
//         self.camera_path = path;
//     }
// }

// pub struct BridgeSender {
//     sender: Sender<BridgeData>,
// }
// impl BridgeSender {
//     pub fn new(sender: Sender<BridgeData>) -> Self {
//         Self { sender }
//     }

//     pub fn message(&self, s: String) {
//         let data = BridgeData::Message(s);
//         self.sender.send(data).unwrap();
//     }

//     pub fn time_code_range(&self, start: f64, end: f64) {
//         let data = BridgeData::TimeCodeRange(start, end);
//         self.sender.send(data).unwrap();
//     }

//     pub fn transform_matrix(&self, path: String, matrix: &[f64]) {
//         let mut data = [0.0; 16];
//         for i in 0..16 {
//             data[i] = matrix[i] as f32;
//         }
//         let data = BridgeData::TransformMatrix(UsdSdfPath(path), data);
//         self.sender.send(data).unwrap();
//     }

//     pub fn create_mesh(&self, path: String) {
//         let data = BridgeData::CreateMesh(UsdSdfPath(path));
//         self.sender.send(data).unwrap();
//     }

//     pub fn mesh_data(&self, path: String, data: Box<MeshData>) {
//         let data = BridgeData::MeshData(UsdSdfPath(path), data.into());
//         self.sender.send(data).unwrap();
//     }

//     pub fn destroy_mesh(&self, path: String) {
//         let data = BridgeData::DestroyMesh(UsdSdfPath(path));
//         self.sender.send(data).unwrap();
//     }

//     pub fn create_distant_light(&self, path: String) {
//         let data = BridgeData::CreateDistantLight(UsdSdfPath(path));
//         self.sender.send(data).unwrap();
//     }

//     pub fn distant_light_data(&self, path: String, data: Box<DistantLightData>) {
//         let data = BridgeData::DistantLightData(UsdSdfPath(path), data.into());
//         self.sender.send(data).unwrap();
//     }

//     pub fn destroy_distant_light(&self, path: String) {
//         let data = BridgeData::DestroyDistantLight(UsdSdfPath(path));
//         self.sender.send(data).unwrap();
//     }

//     pub fn create_sphere_light(&self, path: String) {
//         let data = BridgeData::CreateSphereLight(UsdSdfPath(path));
//         self.sender.send(data).unwrap();
//     }

//     pub fn sphere_light_data(&self, path: String, data: Box<SphereLightData>) {
//         let data = BridgeData::SphereLightData(UsdSdfPath(path), data.into());
//         self.sender.send(data).unwrap();
//     }

//     pub fn destroy_sphere_light(&self, path: String) {
//         let data = BridgeData::DestroySphereLight(UsdSdfPath(path));
//         self.sender.send(data).unwrap();
//     }

//     pub fn create_camera(&self, path: String) {
//         let data = BridgeData::CreateCamera(UsdSdfPath(path));
//         self.sender.send(data).unwrap();
//     }

//     pub fn camera_data(&self, path: String, data: Box<CameraData>) {
//         let data = BridgeData::CameraData(UsdSdfPath(path), data.into());
//         self.sender.send(data).unwrap();
//     }

//     pub fn destroy_camera(&self, path: String) {
//         let data = BridgeData::DestroyCamera(UsdSdfPath(path));
//         self.sender.send(data).unwrap();
//     }

//     pub fn create_render_settings(&self, path: String) {
//         let data = BridgeData::CreateRenderSettings(UsdSdfPath(path));
//         self.sender.send(data).unwrap();
//     }

//     pub fn render_settings_data(&self, path: String, data: Box<RenderSettingsData>) {
//         let data = BridgeData::RenderSettingsData(UsdSdfPath(path), data.into());
//         self.sender.send(data).unwrap();
//     }

//     pub fn destroy_render_settings(&self, path: String) {
//         let data = BridgeData::DestroyRenderSettings(UsdSdfPath(path));
//         self.sender.send(data).unwrap();
//     }

//     pub fn create_render_product(&self, path: String) {
//         let data = BridgeData::CreateRenderProduct(UsdSdfPath(path));
//         self.sender.send(data).unwrap();
//     }

//     pub fn render_product_data(&self, path: String, data: Box<RenderProductData>) {
//         let data = BridgeData::RenderProductData(UsdSdfPath(path), data.into());
//         self.sender.send(data).unwrap();
//     }

//     pub fn destroy_render_product(&self, path: String) {
//         let data = BridgeData::DestroyRenderProduct(UsdSdfPath(path));
//         self.sender.send(data).unwrap();
//     }
// }
