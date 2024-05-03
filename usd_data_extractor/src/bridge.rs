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
        type UsdDataDiff;

        // meshが生成されたdiffの記録とそのデータを設定する関数
        fn create_mesh(&mut self, path: String);
        fn create_mesh_transform_matrix(&mut self, path: String, matrix: &[f32]);
        fn create_mesh_left_handed(&mut self, path: String, left_handed: bool);
        fn create_mesh_points(&mut self, path: String, data: &[f32]);
        fn create_mesh_normals(&mut self, path: String, data: &[f32]);
        fn create_mesh_normals_interpolation(&mut self, path: String, interpolation: Interpolation);
        fn create_mesh_uvs(&mut self, path: String, data: &[f32]);
        fn create_mesh_uvs_indices(&mut self, path: String, data: &[u32]);
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
        fn create_mesh_geom_subset_material_binding(
            &mut self,
            path: String,
            name: String,
            material_path: String,
        );
        fn create_mesh_material_binding(&mut self, path: String, material_path: String);

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
        fn diff_mesh_data_uvs_indices(&mut self, path: String, data: &[u32]);
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
        fn diff_mesh_data_geom_subset_material_binding(
            &mut self,
            path: String,
            name: String,
            material_path: String,
        );
        fn diff_mesh_material_binding(&mut self, path: String, material_path: String);

        // sphere lightが生成/更新されたdiffの記録とそのデータを設定する関数
        fn add_or_update_sphere_light(&mut self, path: String);
        fn add_or_update_sphere_light_transform_matrix(&mut self, path: String, matrix: &[f32]);
        fn add_or_update_sphere_light_color(&mut self, path: String, r: f32, g: f32, b: f32);
        fn add_or_update_sphere_light_intensity(&mut self, path: String, intensity: f32);
        fn add_or_update_sphere_light_cone_angle(&mut self, path: String, angle: f32);
        fn add_or_update_sphere_light_cone_softness(&mut self, path: String, softness: f32);

        // sphere lightが削除されたdiffを記録する関数
        fn destroy_sphere_light(&mut self, path: String);

        // distant lightが生成/更新されたdiffの記録とそのデータを設定する関数
        fn add_or_update_distant_light(&mut self, path: String);
        fn add_or_update_distant_light_transform_matrix(&mut self, path: String, matrix: &[f32]);
        fn add_or_update_distant_light_color(&mut self, path: String, r: f32, g: f32, b: f32);
        fn add_or_update_distant_light_intensity(&mut self, path: String, intensity: f32);

        // distant lightが削除されたdiffを記録する関数
        fn destroy_distant_light(&mut self, path: String);

        // cameraが生成/更新されたdiffの記録とそのデータを設定する関数
        fn add_or_update_camera(&mut self, path: String);
        fn add_or_update_camera_transform_matrix(&mut self, path: String, matrix: &[f32]);
        fn add_or_update_camera_focal_length(&mut self, path: String, fov: f32);
        fn add_or_update_camera_vertical_aperture(&mut self, path: String, aperture: f32);

        // cameraが削除されたdiffを記録する関数
        fn destroy_camera(&mut self, path: String);

        // render settingsが生成/更新されたdiffの記録とそのデータを設定する関数
        fn add_or_update_render_settings(&mut self, path: String);
        fn add_or_update_render_settings_render_product(
            &mut self,
            path: String,
            product_path: String,
            camera_path: String,
        );

        // render settingsが削除されたdiffを記録する関数
        fn destroy_render_settings(&mut self, path: String);

        // materialが生成/更新されたdiffの記録とそのデータを設定する関数
        fn add_or_update_material(&mut self, path: String);
        fn add_or_update_material_diffuse_color(&mut self, path: String, r: f32, g: f32, b: f32);
        fn add_or_update_material_emissive(&mut self, path: String, r: f32, g: f32, b: f32);
        fn add_or_update_material_metallic(&mut self, path: String, metallic: f32);
        fn add_or_update_material_opacity(&mut self, path: String, opacity: f32);
        fn add_or_update_material_roughness(&mut self, path: String, roughness: f32);
        fn add_or_update_material_diffuse_color_file(&mut self, path: String, file_path: String);
        fn add_or_update_material_emissive_file(&mut self, path: String, file_path: String);
        fn add_or_update_material_metallic_file(&mut self, path: String, file_path: String);
        fn add_or_update_material_normal_file(&mut self, path: String, file_path: String);
        fn add_or_update_material_opacity_file(&mut self, path: String, file_path: String);
        fn add_or_update_material_roughness_file(&mut self, path: String, file_path: String);

        // materialが削除されたdiffを記録する関数
        fn destroy_material(&mut self, path: String);
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

#[derive(Debug, Default, Hash, PartialEq, Eq)]
pub struct SdfPath(String);
impl Into<String> for SdfPath {
    fn into(self) -> String {
        self.0
    }
}

#[derive(Debug, Default)]
pub struct SubMeshData {
    pub indices_type: String,
    pub indices: Vec<u32>,
    pub material_path: Option<String>,
}

#[derive(Debug, Default)]
pub struct MeshCreate {
    pub transform_matrix: Option<[f32; 16]>,
    pub left_handed: Option<bool>,
    pub points: Option<Vec<f32>>,
    pub normals: Option<Vec<f32>>,
    pub normals_interpolation: Option<Interpolation>,
    pub uvs: Option<Vec<f32>>,
    pub uvs_indices: Option<Vec<u32>>,
    pub uvs_interpolation: Option<Interpolation>,
    pub face_vertex_indices: Option<Vec<u32>>,
    pub face_vertex_counts: Option<Vec<u32>>,
    pub geom_subsets: HashMap<String, SubMeshData>,
    pub material_path: Option<String>,
}

#[derive(Debug, Default)]
pub struct MeshDataDiff {
    pub left_handed: Option<bool>,
    pub points: Option<Vec<f32>>,
    pub normals: Option<Vec<f32>>,
    pub normals_interpolation: Option<Interpolation>,
    pub uvs: Option<Vec<f32>>,
    pub uvs_indices: Option<Vec<u32>>,
    pub uvs_interpolation: Option<Interpolation>,
    pub face_vertex_indices: Option<Vec<u32>>,
    pub face_vertex_counts: Option<Vec<u32>>,
    pub geom_subsets: HashMap<String, SubMeshData>,
    pub material_path: Option<String>,
}

#[derive(Debug, Default)]
pub struct MeshesDiff {
    pub create: HashMap<SdfPath, MeshCreate>,
    pub destroy: Vec<SdfPath>,
    pub diff_transform_matrix: HashMap<SdfPath, [f32; 16]>,
    pub diff_mesh_data: HashMap<SdfPath, MeshDataDiff>,
}

#[derive(Debug, Default)]
pub struct SphereLightData {
    pub transform_matrix: Option<[f32; 16]>,
    pub color: Option<[f32; 3]>,
    pub intensity: Option<f32>,
    pub cone_angle: Option<f32>,
    pub cone_softness: Option<f32>,
}

#[derive(Debug, Default)]
pub struct SphereLightsDiff {
    pub update: HashMap<SdfPath, SphereLightData>,
    pub destroy: Vec<SdfPath>,
}

#[derive(Debug, Default)]
pub struct DistantLightData {
    pub transform_matrix: Option<[f32; 16]>,
    pub color: Option<[f32; 3]>,
    pub intensity: Option<f32>,
}

#[derive(Debug, Default)]
pub struct DistantLightsDiff {
    pub update: HashMap<SdfPath, DistantLightData>,
    pub destroy: Vec<SdfPath>,
}

#[derive(Debug, Default)]
pub struct CameraData {
    pub transform_matrix: Option<[f32; 16]>,
    pub focal_length: Option<f32>,
    pub vertical_aperture: Option<f32>,
}

#[derive(Debug, Default)]
pub struct CamerasDiff {
    pub update: HashMap<SdfPath, CameraData>,
    pub destroy: Vec<SdfPath>,
}

#[derive(Debug, Default)]
pub struct RenderProductData {
    pub camera_path: SdfPath,
}

#[derive(Debug, Default)]
pub struct RenderSettingsData {
    pub render_product: HashMap<String, RenderProductData>,
}

#[derive(Debug, Default)]
pub struct RenderSettingsDiff {
    pub update: HashMap<SdfPath, RenderSettingsData>,
    pub destroy: Vec<SdfPath>,
}

#[derive(Debug, Default)]
pub struct MaterialDiffItem {
    pub diffuse_color: Option<[f32; 3]>,
    pub emissive: Option<[f32; 3]>,
    pub metallic: Option<f32>,
    pub opacity: Option<f32>,
    pub roughness: Option<f32>,
    pub diffuse_color_file: Option<String>,
    pub emissive_file: Option<String>,
    pub metallic_file: Option<String>,
    pub normal_file: Option<String>,
    pub opacity_file: Option<String>,
    pub roughness_file: Option<String>,
}

#[derive(Debug, Default)]
pub struct MaterialsDiff {
    pub update: HashMap<SdfPath, MaterialDiffItem>,
    pub destroy: Vec<SdfPath>,
}

#[derive(Debug, Default)]
pub struct UsdDataDiff {
    pub meshes: MeshesDiff,
    pub sphere_lights: SphereLightsDiff,
    pub distant_lights: DistantLightsDiff,
    pub cameras: CamerasDiff,
    pub render_settings: RenderSettingsDiff,
    pub materials: MaterialsDiff,
}
impl UsdDataDiff {
    // === Mesh ===

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

    fn create_mesh_uvs_indices(&mut self, path: String, data: &[u32]) {
        if let Some(create) = self.meshes.create.get_mut(&SdfPath(path)) {
            create.uvs_indices = Some(data.to_vec());
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
            let sub_mesh = create.geom_subsets.entry(name).or_default();
            sub_mesh.indices_type = ty_;
            sub_mesh.indices = indices.to_vec();
        }
    }

    fn create_mesh_geom_subset_material_binding(
        &mut self,
        path: String,
        name: String,
        material_path: String,
    ) {
        if let Some(create) = self.meshes.create.get_mut(&SdfPath(path)) {
            let sub_mesh = create.geom_subsets.entry(name).or_default();
            sub_mesh.material_path = Some(material_path);
        }
    }

    fn create_mesh_material_binding(&mut self, path: String, material_path: String) {
        if let Some(create) = self.meshes.create.get_mut(&SdfPath(path)) {
            create.material_path = Some(material_path);
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

    fn diff_mesh_data_uvs_indices(&mut self, path: String, data: &[u32]) {
        if let Some(diff) = self.meshes.diff_mesh_data.get_mut(&SdfPath(path)) {
            diff.uvs_indices = Some(data.to_vec());
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
            let sub_mesh = diff.geom_subsets.entry(name).or_default();
            sub_mesh.indices_type = ty_;
            sub_mesh.indices = indices.to_vec();
        }
    }

    fn diff_mesh_data_geom_subset_material_binding(
        &mut self,
        path: String,
        name: String,
        material_path: String,
    ) {
        if let Some(diff) = self.meshes.diff_mesh_data.get_mut(&SdfPath(path)) {
            let sub_mesh = diff.geom_subsets.entry(name).or_default();
            sub_mesh.material_path = Some(material_path);
        }
    }

    fn diff_mesh_material_binding(&mut self, path: String, material_path: String) {
        if let Some(diff) = self.meshes.diff_mesh_data.get_mut(&SdfPath(path)) {
            diff.material_path = Some(material_path);
        }
    }

    // === Sphere Light ===

    fn add_or_update_sphere_light(&mut self, path: String) {
        self.sphere_lights
            .update
            .insert(SdfPath(path), SphereLightData::default());
    }

    fn add_or_update_sphere_light_transform_matrix(&mut self, path: String, matrix: &[f32]) {
        let data = matrix[0..16].try_into().unwrap();
        if let Some(create) = self.sphere_lights.update.get_mut(&SdfPath(path)) {
            create.transform_matrix = Some(data);
        }
    }

    fn add_or_update_sphere_light_color(&mut self, path: String, r: f32, g: f32, b: f32) {
        if let Some(create) = self.sphere_lights.update.get_mut(&SdfPath(path)) {
            create.color = Some([r, g, b]);
        }
    }

    fn add_or_update_sphere_light_intensity(&mut self, path: String, intensity: f32) {
        if let Some(create) = self.sphere_lights.update.get_mut(&SdfPath(path)) {
            create.intensity = Some(intensity);
        }
    }

    fn add_or_update_sphere_light_cone_angle(&mut self, path: String, angle: f32) {
        if let Some(create) = self.sphere_lights.update.get_mut(&SdfPath(path)) {
            create.cone_angle = Some(angle);
        }
    }

    fn add_or_update_sphere_light_cone_softness(&mut self, path: String, softness: f32) {
        if let Some(create) = self.sphere_lights.update.get_mut(&SdfPath(path)) {
            create.cone_softness = Some(softness);
        }
    }

    fn destroy_sphere_light(&mut self, path: String) {
        self.sphere_lights.destroy.push(SdfPath(path));
    }

    // === Distant Light ===

    fn add_or_update_distant_light(&mut self, path: String) {
        self.distant_lights
            .update
            .insert(SdfPath(path), DistantLightData::default());
    }

    fn add_or_update_distant_light_transform_matrix(&mut self, path: String, matrix: &[f32]) {
        let data = matrix[0..16].try_into().unwrap();
        if let Some(create) = self.distant_lights.update.get_mut(&SdfPath(path)) {
            create.transform_matrix = Some(data);
        }
    }

    fn add_or_update_distant_light_color(&mut self, path: String, r: f32, g: f32, b: f32) {
        if let Some(create) = self.distant_lights.update.get_mut(&SdfPath(path)) {
            create.color = Some([r, g, b]);
        }
    }

    fn add_or_update_distant_light_intensity(&mut self, path: String, intensity: f32) {
        if let Some(create) = self.distant_lights.update.get_mut(&SdfPath(path)) {
            create.intensity = Some(intensity);
        }
    }

    fn destroy_distant_light(&mut self, path: String) {
        self.distant_lights.destroy.push(SdfPath(path));
    }

    // === Camera ===

    fn add_or_update_camera(&mut self, path: String) {
        self.cameras
            .update
            .insert(SdfPath(path), CameraData::default());
    }

    fn add_or_update_camera_transform_matrix(&mut self, path: String, matrix: &[f32]) {
        let data = matrix[0..16].try_into().unwrap();
        if let Some(create) = self.cameras.update.get_mut(&SdfPath(path)) {
            create.transform_matrix = Some(data);
        }
    }

    fn add_or_update_camera_focal_length(&mut self, path: String, fov: f32) {
        if let Some(create) = self.cameras.update.get_mut(&SdfPath(path)) {
            create.focal_length = Some(fov);
        }
    }

    fn add_or_update_camera_vertical_aperture(&mut self, path: String, aperture: f32) {
        if let Some(create) = self.cameras.update.get_mut(&SdfPath(path)) {
            create.vertical_aperture = Some(aperture);
        }
    }

    fn destroy_camera(&mut self, path: String) {
        self.cameras.destroy.push(SdfPath(path));
    }

    // === Render Settings ===

    fn add_or_update_render_settings(&mut self, path: String) {
        self.render_settings
            .update
            .insert(SdfPath(path), RenderSettingsData::default());
    }

    fn add_or_update_render_settings_render_product(
        &mut self,
        path: String,
        product_path: String,
        camera_path: String,
    ) {
        if let Some(create) = self.render_settings.update.get_mut(&SdfPath(path)) {
            create.render_product.insert(
                product_path,
                RenderProductData {
                    camera_path: SdfPath(camera_path),
                },
            );
        }
    }

    fn destroy_render_settings(&mut self, path: String) {
        self.render_settings.destroy.push(SdfPath(path));
    }

    // === Material ===

    fn add_or_update_material(&mut self, path: String) {
        self.materials
            .update
            .insert(SdfPath(path), MaterialDiffItem::default());
    }

    fn add_or_update_material_diffuse_color(&mut self, path: String, r: f32, g: f32, b: f32) {
        if let Some(create) = self.materials.update.get_mut(&SdfPath(path)) {
            create.diffuse_color = Some([r, g, b]);
        }
    }

    fn add_or_update_material_emissive(&mut self, path: String, r: f32, g: f32, b: f32) {
        if let Some(create) = self.materials.update.get_mut(&SdfPath(path)) {
            create.emissive = Some([r, g, b]);
        }
    }

    fn add_or_update_material_metallic(&mut self, path: String, metallic: f32) {
        if let Some(create) = self.materials.update.get_mut(&SdfPath(path)) {
            create.metallic = Some(metallic);
        }
    }

    fn add_or_update_material_opacity(&mut self, path: String, opacity: f32) {
        if let Some(create) = self.materials.update.get_mut(&SdfPath(path)) {
            create.opacity = Some(opacity);
        }
    }

    fn add_or_update_material_roughness(&mut self, path: String, roughness: f32) {
        if let Some(create) = self.materials.update.get_mut(&SdfPath(path)) {
            create.roughness = Some(roughness);
        }
    }

    fn add_or_update_material_diffuse_color_file(&mut self, path: String, file_path: String) {
        if let Some(create) = self.materials.update.get_mut(&SdfPath(path)) {
            create.diffuse_color_file = Some(file_path);
        }
    }

    fn add_or_update_material_emissive_file(&mut self, path: String, file_path: String) {
        if let Some(create) = self.materials.update.get_mut(&SdfPath(path)) {
            create.emissive_file = Some(file_path);
        }
    }

    fn add_or_update_material_metallic_file(&mut self, path: String, file_path: String) {
        if let Some(create) = self.materials.update.get_mut(&SdfPath(path)) {
            create.metallic_file = Some(file_path);
        }
    }

    fn add_or_update_material_normal_file(&mut self, path: String, file_path: String) {
        if let Some(create) = self.materials.update.get_mut(&SdfPath(path)) {
            create.normal_file = Some(file_path);
        }
    }

    fn add_or_update_material_opacity_file(&mut self, path: String, file_path: String) {
        if let Some(create) = self.materials.update.get_mut(&SdfPath(path)) {
            create.opacity_file = Some(file_path);
        }
    }

    fn add_or_update_material_roughness_file(&mut self, path: String, file_path: String) {
        if let Some(create) = self.materials.update.get_mut(&SdfPath(path)) {
            create.roughness_file = Some(file_path);
        }
    }

    fn destroy_material(&mut self, path: String) {
        self.materials.destroy.push(SdfPath(path));
    }
}
