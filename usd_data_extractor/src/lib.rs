use std::fmt::Display;
use std::path::Path;
use std::sync::mpsc::Receiver;

mod bridge;

#[derive(Debug, Clone)]
pub enum Interpolation {
    Constant,
    Uniform,
    Varying,
    Vertex,
    FaceVarying,
    Instance,
}
impl From<u8> for Interpolation {
    fn from(i: u8) -> Self {
        match i {
            0 => Interpolation::Constant,
            1 => Interpolation::Uniform,
            2 => Interpolation::Varying,
            3 => Interpolation::Vertex,
            4 => Interpolation::FaceVarying,
            5 => Interpolation::Instance,
            _ => panic!("Invalid interpolation value: {}", i),
        }
    }
}

#[derive(Debug, Clone)]
pub struct MeshData {
    pub left_handed: bool,
    pub points_data: Vec<f32>,
    pub points_interpolation: Interpolation,
    pub normals_data: Option<Vec<f32>>,
    pub normals_interpolation: Option<Interpolation>,
    pub uvs_data: Option<Vec<f32>>,
    pub uvs_interpolation: Option<Interpolation>,
    pub face_vertex_indices: Vec<u64>,
    pub face_vertex_counts: Vec<u32>,
}
impl From<Box<bridge::MeshData>> for MeshData {
    fn from(data: Box<bridge::MeshData>) -> Self {
        Self {
            left_handed: data.left_handed,
            points_data: data.points_data.expect("MeshData has no points data"),
            points_interpolation: data
                .points_interpolation
                .expect("MeshData has no points data"),
            normals_data: data.normals_data,
            normals_interpolation: data.normals_interpolation,
            uvs_data: data.uvs_data,
            uvs_interpolation: data.uvs_interpolation,
            face_vertex_indices: data
                .face_vertex_indices
                .expect("MeshData has no face vertex indices data"),
            face_vertex_counts: data
                .face_vertex_counts
                .expect("MeshData has no face vertex count data"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct DistantLightData {
    pub intensity: f32,
    pub color: [f32; 3],
}
impl From<Box<bridge::DistantLightData>> for DistantLightData {
    fn from(data: Box<bridge::DistantLightData>) -> Self {
        Self {
            intensity: data.intensity,
            color: data.color,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SphereLightData {
    pub intensity: f32,
    pub color: [f32; 3],
    pub radius: f32,
    pub cone_angle: Option<f32>,
    pub cone_softness: Option<f32>,
}
impl From<Box<bridge::SphereLightData>> for SphereLightData {
    fn from(data: Box<bridge::SphereLightData>) -> Self {
        Self {
            intensity: data.intensity,
            color: data.color,
            radius: data.radius,
            cone_angle: data.cone_angle,
            cone_softness: data.cone_softness,
        }
    }
}

#[derive(Debug, Clone)]
pub struct UsdSdfPath(pub String);
impl Display for UsdSdfPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub enum BridgeData {
    Message(String),
    TimeCodeRange(f64, f64),
    TransformMatrix(UsdSdfPath, [f32; 16]),
    CreateMesh(UsdSdfPath),
    MeshData(UsdSdfPath, MeshData),
    DestroyMesh(UsdSdfPath),
    CreateDistantLight(UsdSdfPath),
    DistantLightData(UsdSdfPath, DistantLightData),
    DestroyDistantLight(UsdSdfPath),
    CreateSphereLight(UsdSdfPath),
    SphereLightData(UsdSdfPath, SphereLightData),
    DestroySphereLight(UsdSdfPath),
}

pub struct UsdDataExtractor {
    inner: cxx::UniquePtr<bridge::ffi::BridgeUsdDataExtractor>,
    rx: Receiver<BridgeData>,
}
impl UsdDataExtractor {
    pub fn new(path: impl AsRef<Path>) -> Result<Self, String> {
        let (tx, rx) = std::sync::mpsc::channel();
        let sender = Box::new(bridge::BridgeSender::new(tx));
        let inner = bridge::ffi::new_usd_data_extractor(sender, path.as_ref().to_str().unwrap())
            .map_err(|e| String::from(e.what()))?;
        Ok(Self { inner, rx })
    }

    pub fn extract(&mut self, time_code: f64) -> Vec<BridgeData> {
        let (notifier, rx) = bridge::BridgeSendEndNotifier::new();
        let inner = self.inner.pin_mut();
        inner.extract(notifier, time_code);
        let _ = rx.recv();

        let mut data = vec![];
        while let Ok(d) = self.rx.try_recv() {
            data.push(d);
        }
        data
    }

    pub fn destroy(self) -> Vec<BridgeData> {
        drop(self.inner);
        let mut data = vec![];
        while let Ok(d) = self.rx.try_recv() {
            data.push(d);
        }
        data
    }
}
