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
pub struct UsdSdfPath(pub String);
impl Display for UsdSdfPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub enum BridgeData {
    Message(String),
    TimeCodeRange(f64, f64),
    CreateMesh(UsdSdfPath),
    TransformMatrix(UsdSdfPath, [f32; 16]),
    Points(UsdSdfPath, Vec<f32>, Interpolation),
    Normals(UsdSdfPath, Vec<f32>, Interpolation),
    Uvs(UsdSdfPath, Vec<f32>, Interpolation),
    Indices(UsdSdfPath, Vec<u32>),
    FaceVertexCount(UsdSdfPath, Vec<u8>),
    DestroyMesh(UsdSdfPath),
}

pub struct UsdDataExtractor {
    inner: cxx::UniquePtr<bridge::ffi::BridgeUsdDataExtractor>,
    rx: Receiver<BridgeData>,
}
impl UsdDataExtractor {
    pub fn new(path: impl AsRef<Path>) -> Self {
        let (tx, rx) = std::sync::mpsc::channel();
        let sender = Box::new(bridge::BridgeSender::new(tx));
        let inner = bridge::ffi::new_usd_data_extractor(sender, path.as_ref().to_str().unwrap());
        Self { inner, rx }
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
