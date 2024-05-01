use glam::{Mat4, Vec2, Vec3};
use std::collections::HashMap;
use std::path::Path;

mod bridge;

pub use bridge::{Interpolation, SdfPath};

/// USDから抽出したシーンのtransform matrixの情報
#[derive(Debug)]
pub struct TransformMatrix {
    pub matrix: Mat4,
}

/// 頂点バッファの一つの頂点情報
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct Vertex {
    pub position: Vec3,
    pub normal: Vec3,
    pub uv: Vec2,
}

// sub meshのindex情報
#[derive(Debug)]
pub struct SubMesh {
    /// sub meshのindices
    pub indices: Vec<u32>,
}

/// USDから抽出した頂点属性などをduplicateしtriangulateし、sub meshに分割したデータ。
/// 頂点バッファのデータと、sub meshのindex情報を持つ。
#[derive(Debug)]
pub struct MeshData {
    /// 頂点バッファのデータ
    pub vertices: Vec<Vertex>,
    /// sub meshのindex情報
    pub sub_meshes: Vec<SubMesh>,
}
impl MeshData {
    fn new(
        left_handed: bool,
        points: Vec<f32>,
        normals: Option<Vec<f32>>,
        normals_interpolation: Option<Interpolation>,
        uvs: Option<Vec<f32>>,
        uvs_interpolation: Option<Interpolation>,
        face_vertex_indices: Vec<u32>,
        face_vertex_counts: Vec<u32>,
        geom_subsets: HashMap<String, (String, Vec<u32>)>,
    ) -> Self {
        // InterpolationがVertexの頂点データをduplicatedするindexを計算する
        let duplicate_vertex_indices = {
            let mut vertex_indices = Vec::new();
            for i in 0..face_vertex_indices.len() {
                vertex_indices.push(face_vertex_indices[i] as usize);
            }
            vertex_indices
        };

        // duplicatedした頂点座標のデータを作成する
        // Pointsは常にInterpolationはVertex
        let vertex_points = {
            let points = bytemuck::cast_slice::<f32, Vec3>(&points);
            let mut data = Vec::with_capacity(duplicate_vertex_indices.len());
            for &index in &duplicate_vertex_indices {
                data.push(points[index]);
            }
            data
        };

        // duplicatedした法線ベクトルのデータを作成する
        let vertex_normals = {
            let vertex_normals = if let (Some(normals), Some(normals_interpolation)) =
                (normals, normals_interpolation)
            {
                // データが渡された場合、そのデータのInterpolationに合わせてduplicatedする
                // 現状はFaceVaryingとVertexのInterpolationのみ対応
                match normals_interpolation {
                    Interpolation::FaceVarying => {
                        let normals = bytemuck::cast_slice::<f32, Vec3>(&normals);
                        Some(normals.to_vec())
                    }
                    Interpolation::Vertex => {
                        let normals = bytemuck::cast_slice::<f32, Vec3>(&normals);
                        let mut data = Vec::with_capacity(duplicate_vertex_indices.len());
                        for &index in &duplicate_vertex_indices {
                            data.push(normals[index]);
                        }
                        Some(data)
                    }
                    _ => None,
                }
            } else {
                None
            };
            if let Some(vertex_normals) = vertex_normals {
                vertex_normals
            } else {
                // Interpolationが対応していない場合や、頂点法線データが渡されていない場合、頂点データから計算する
                let points = bytemuck::cast_slice::<f32, Vec3>(&points);
                let mut normals_point = vec![Vec3::ZERO; points.len()];
                let indices = &duplicate_vertex_indices;
                let mut index_offset = 0;
                for face_vertex_count in &face_vertex_counts {
                    let face_vertex_count = *face_vertex_count as usize;
                    for i in 2..face_vertex_count {
                        let p0 = points[indices[index_offset]];
                        let p1 = points[indices[index_offset + i - 1]];
                        let p2 = points[indices[index_offset + i]];
                        let mut normal = (p1 - p0).cross(p2 - p0).normalize();
                        if left_handed {
                            normal = -normal;
                        }
                        for &index in &indices[index_offset..index_offset + face_vertex_count] {
                            normals_point[index] += normal;
                        }
                    }
                    index_offset += face_vertex_count;
                }
                let mut mean_normals = Vec::with_capacity(points.len());
                for normals in normals_point {
                    mean_normals.push(normals.normalize());
                }
                let mut vertex_normals = Vec::with_capacity(indices.len());
                for &index in indices {
                    vertex_normals.push(mean_normals[index]);
                }
                vertex_normals
            }
        };

        // duplicatedしたUV座標のデータを作成する
        let vertex_uvs = {
            if let (Some(uvs), Some(uvs_interpolation)) = (uvs, uvs_interpolation) {
                match uvs_interpolation {
                    Interpolation::FaceVarying => {
                        let uvs = bytemuck::cast_slice::<f32, Vec2>(&uvs);
                        Some(uvs.to_vec())
                    }
                    Interpolation::Vertex => {
                        let uvs = bytemuck::cast_slice::<f32, Vec2>(&uvs);
                        let mut data = Vec::with_capacity(duplicate_vertex_indices.len());
                        for &index in &duplicate_vertex_indices {
                            data.push(uvs[index]);
                        }
                        Some(data)
                    }
                    _ => None,
                }
            } else {
                None
            }
        };

        // vertex bufferの情報を作る
        let mut vertices = Vec::with_capacity(vertex_points.len());
        for i in 0..vertex_points.len() {
            vertices.push(Vertex {
                position: vertex_points[i],
                normal: vertex_normals[i],
                uv: vertex_uvs.as_ref().map_or(Vec2::ZERO, |uvs| uvs[i]),
            });
        }

        // sub meshのindex情報を作る
        let mut used_face_indices = Vec::new();
        let mut sub_meshes = Vec::new();
        for (_sub_mesh_name, (ty_, indices)) in geom_subsets {
            // typeFaceSet以外のいgeomSubsetsには未対応
            if ty_ != "typeFaceSet" {
                continue;
            }

            // geomSubsetに含まれるfaceをtriangulateしたindexを作る
            let mut sub_mesh_indices = Vec::new();
            let mut index_offset = 0;
            for (i, face_vertex_count) in face_vertex_counts.iter().enumerate() {
                let face_vertex_count = *face_vertex_count as usize;
                if indices.contains(&(i as u32)) {
                    for i in 2..face_vertex_count {
                        if left_handed {
                            sub_mesh_indices.push((index_offset + i) as u32);
                            sub_mesh_indices.push((index_offset + i - 1) as u32);
                            sub_mesh_indices.push(index_offset as u32);
                        } else {
                            sub_mesh_indices.push(index_offset as u32);
                            sub_mesh_indices.push((index_offset + i - 1) as u32);
                            sub_mesh_indices.push((index_offset + i) as u32);
                        }
                    }
                    used_face_indices.push(i as u64);
                }
                index_offset += face_vertex_count;
            }
            sub_meshes.push(SubMesh {
                indices: sub_mesh_indices,
            });
        }

        // geomSubsetに含まれなかったfaceをtriangulateしたindexによるsub meshを作る
        let mut base_indices = Vec::new();
        for i in 0..face_vertex_counts.len() {
            if !used_face_indices.contains(&(i as u64)) {
                base_indices.push(i as u64);
            }
        }
        let mut sub_mesh_indices = Vec::new();
        let mut index_offset = 0;
        for (i, face_vertex_count) in face_vertex_counts.iter().enumerate() {
            let face_vertex_count = *face_vertex_count as usize;
            if base_indices.contains(&(i as u64)) {
                for i in 2..face_vertex_count {
                    if left_handed {
                        sub_mesh_indices.push((index_offset + i) as u32);
                        sub_mesh_indices.push((index_offset + i - 1) as u32);
                        sub_mesh_indices.push(index_offset as u32);
                    } else {
                        sub_mesh_indices.push(index_offset as u32);
                        sub_mesh_indices.push((index_offset + i - 1) as u32);
                        sub_mesh_indices.push((index_offset + i) as u32);
                    }
                }
                used_face_indices.push(i as u64);
            }
            index_offset += face_vertex_count;
        }
        sub_meshes.push(SubMesh {
            indices: sub_mesh_indices,
        });

        Self {
            vertices,
            sub_meshes,
        }
    }
}

/// USDから抽出したシーンのSphereLightの情報
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
impl SphereLight {
    fn new(
        transform_matrix: Mat4,
        intensity: f32,
        color: Vec3,
        cone_angle: Option<f32>,
        cone_softness: Option<f32>,
    ) -> Self {
        let position = transform_matrix.transform_point3(Vec3::ZERO);
        if let (Some(cone_angle), Some(cone_softness)) = (cone_angle, cone_softness) {
            let direction = transform_matrix.transform_vector3(Vec3::Z);
            Self {
                is_spot: true,
                position,
                intensity,
                color,
                direction: Some(direction),
                cone_angle: Some(cone_angle),
                cone_softness: Some(cone_softness),
            }
        } else {
            Self {
                is_spot: false,
                position,
                intensity,
                color,
                direction: None,
                cone_angle: None,
                cone_softness: None,
            }
        }
    }
}

/// USDから抽出したシーンのDistantLightの情報
#[derive(Debug)]
pub struct DistantLight {
    pub direction: Vec3,
    pub intensity: f32,
    pub color: Vec3,
}
impl DistantLight {
    fn new(transform_matrix: Mat4, intensity: f32, color: Vec3) -> Self {
        let direction = transform_matrix.transform_vector3(Vec3::Z);
        Self {
            direction,
            intensity,
            color,
        }
    }
}

/// USDから抽出したシーンのCameraの情報
#[derive(Debug, Clone)]
pub struct Camera {
    pub eye: Vec3,
    pub dir: Vec3,
    pub fovy: f32,
}
impl Camera {
    fn new(transform_matrix: Mat4, focal_length: f32, vertical_aperture: f32) -> Self {
        let eye = transform_matrix.transform_point3(Vec3::ZERO);
        let dir = transform_matrix.transform_vector3(Vec3::NEG_Z);
        let fovy = 2.0 * (vertical_aperture / (2.0 * focal_length)).atan();
        Self { eye, dir, fovy }
    }
}

#[derive(Debug)]
pub struct RenderProduct {
    pub camera_path: String,
}

#[derive(Debug)]
pub struct RenderSettings {
    pub render_products: HashMap<String, RenderProduct>,
}

/// シーンの変更点の差分情報の一つの要素
pub enum SceneDiffItem {
    MeshCreated(SdfPath, TransformMatrix, MeshData),
    MeshDestroyed(SdfPath),
    MeshTransformMatrixDirtied(SdfPath, TransformMatrix),
    MeshDataDirtied(SdfPath, MeshData),
    SphereLightAddOrUpdate(SdfPath, SphereLight),
    SphereLightDestroyed(SdfPath),
    DistantLightAddOrUpdate(SdfPath, DistantLight),
    DistantLightDestroyed(SdfPath),
    CameraAddOrUpdate(SdfPath, Camera),
    CameraDestroyed(SdfPath),
    RenderSettingsAddOrUpdate(SdfPath, RenderSettings),
    RenderSettingsDestroyed(SdfPath),
}

/// シーンの変更点の差分情報全体
pub struct SceneDiff {
    /// シーンの変更点の差分情報の要素のリスト
    pub items: Vec<SceneDiffItem>,
}
impl From<bridge::UsdDataDiff> for SceneDiff {
    fn from(diff: bridge::UsdDataDiff) -> Self {
        let mut items = Vec::new();

        for (path, data) in diff.meshes.create {
            items.push(SceneDiffItem::MeshCreated(
                path,
                TransformMatrix {
                    matrix: data
                        .transform_matrix
                        .map_or(Mat4::IDENTITY, |data| Mat4::from_cols_array(&data)),
                },
                MeshData::new(
                    data.left_handed.unwrap_or(false),
                    data.points.unwrap(),
                    data.normals,
                    data.normals_interpolation,
                    data.uvs,
                    data.uvs_interpolation,
                    data.face_vertex_indices.unwrap(),
                    data.face_vertex_counts.unwrap(),
                    data.geom_subsets,
                ),
            ));
        }
        for path in diff.meshes.destroy {
            items.push(SceneDiffItem::MeshDestroyed(path));
        }
        for (path, matrix) in diff.meshes.diff_transform_matrix {
            items.push(SceneDiffItem::MeshTransformMatrixDirtied(
                path,
                TransformMatrix {
                    matrix: Mat4::from_cols_array(&matrix),
                },
            ));
        }
        for (path, data) in diff.meshes.diff_mesh_data {
            items.push(SceneDiffItem::MeshDataDirtied(
                path,
                MeshData::new(
                    data.left_handed.unwrap_or(false),
                    data.points.unwrap(),
                    data.normals,
                    data.normals_interpolation,
                    data.uvs,
                    data.uvs_interpolation,
                    data.face_vertex_indices.unwrap(),
                    data.face_vertex_counts.unwrap(),
                    data.geom_subsets,
                ),
            ));
        }

        for (path, data) in diff.sphere_lights.update {
            items.push(SceneDiffItem::SphereLightAddOrUpdate(
                path,
                SphereLight::new(
                    Mat4::from_cols_array(&data.transform_matrix.unwrap()),
                    data.intensity.unwrap(),
                    Vec3::from(data.color.unwrap()),
                    data.cone_angle,
                    data.cone_softness,
                ),
            ));
        }
        for path in diff.sphere_lights.destroy {
            items.push(SceneDiffItem::SphereLightDestroyed(path));
        }

        for (path, data) in diff.distant_lights.update {
            items.push(SceneDiffItem::DistantLightAddOrUpdate(
                path,
                DistantLight::new(
                    Mat4::from_cols_array(&data.transform_matrix.unwrap()),
                    data.intensity.unwrap(),
                    Vec3::from(data.color.unwrap()),
                ),
            ));
        }
        for path in diff.distant_lights.destroy {
            items.push(SceneDiffItem::DistantLightDestroyed(path));
        }

        for (path, data) in diff.cameras.update {
            items.push(SceneDiffItem::CameraAddOrUpdate(
                path,
                Camera::new(
                    Mat4::from_cols_array(&data.transform_matrix.unwrap()),
                    data.focal_length.unwrap(),
                    data.vertical_aperture.unwrap(),
                ),
            ));
        }
        for path in diff.cameras.destroy {
            items.push(SceneDiffItem::CameraDestroyed(path));
        }

        for (path, data) in diff.render_settings.update {
            let mut render_products = HashMap::new();
            for (product_name, product_data) in data.render_product {
                render_products.insert(
                    product_name,
                    RenderProduct {
                        camera_path: product_data.camera_path.into(),
                    },
                );
            }
            items.push(SceneDiffItem::RenderSettingsAddOrUpdate(
                path,
                RenderSettings { render_products },
            ));
        }
        for path in diff.render_settings.destroy {
            items.push(SceneDiffItem::RenderSettingsDestroyed(path));
        }

        Self { items }
    }
}

pub struct UsdSceneExtractor {
    inner: cxx::UniquePtr<bridge::ffi::BridgeUsdDataExtractor>,
    start_time_code: f64,
    end_time_code: f64,
}
impl UsdSceneExtractor {
    pub fn new(path: impl AsRef<Path>) -> Result<Self, String> {
        let path = path.as_ref().to_str().unwrap();
        let inner = bridge::ffi::new_usd_data_extractor(path);
        let inner = inner.map_err(|e| String::from(e.what()))?;
        let start_time_code = inner.start_time_code();
        let end_time_code = inner.end_time_code();
        Ok(Self {
            inner,
            start_time_code,
            end_time_code,
        })
    }

    pub fn time_code_range(&self) -> (f64, f64) {
        (self.start_time_code, self.end_time_code)
    }

    pub fn extract(&mut self, time_code: f64) -> SceneDiff {
        let inner = self.inner.pin_mut();

        let mut usd_data_diff = bridge::UsdDataDiff::default();
        let pin_usd_data_diff = std::pin::Pin::new(&mut usd_data_diff);

        inner.extract(time_code, pin_usd_data_diff);

        usd_data_diff.into()
    }
}
