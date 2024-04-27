use std::sync::mpsc::Sender;

use crate::{BridgeData, Interpolation, UsdSdfPath};

#[cxx::bridge]
pub mod ffi {
    extern "Rust" {
        type MeshData;
        fn new_mesh_data() -> Box<MeshData>;
        fn set_left_handed(self: &mut MeshData, left_handed: bool);
        fn set_points(self: &mut MeshData, data: &[f32], interpolation: u8);
        fn set_normals(self: &mut MeshData, data: &[f32], interpolation: u8);
        fn set_uvs(self: &mut MeshData, data: &[f32], interpolation: u8);
        fn set_face_vertex_indices(self: &mut MeshData, data: &[i32]);
        fn set_face_vertex_counts(self: &mut MeshData, data: &[i32]);

        type BridgeSender;
        fn message(self: &BridgeSender, s: String);
        fn time_code_range(self: &BridgeSender, start: f64, end: f64);
        fn create_mesh(self: &BridgeSender, path: String);
        fn transform_matrix(self: &BridgeSender, path: String, matrix: &[f64]);
        fn mesh_data(self: &BridgeSender, path: String, data: Box<MeshData>);
        fn destroy_mesh(self: &BridgeSender, path: String);

        type BridgeSendEndNotifier;
        fn notify(self: &mut BridgeSendEndNotifier);
    }
    unsafe extern "C++" {
        include!("usd_data_extractor/cpp/usdDataExtractor.h");

        type BridgeUsdDataExtractor;
        fn extract(
            self: Pin<&mut BridgeUsdDataExtractor>,
            notifier: Box<BridgeSendEndNotifier>,
            time_code: f64,
        );
        fn new_usd_data_extractor(
            sender: Box<BridgeSender>,
            open_path: &str,
        ) -> UniquePtr<BridgeUsdDataExtractor>;
    }
}

pub fn new_mesh_data() -> Box<MeshData> {
    Box::new(MeshData {
        left_handed: false,
        points_data: None,
        points_interpolation: None,
        normals_data: None,
        normals_interpolation: None,
        uvs_data: None,
        uvs_interpolation: None,
        face_vertex_indices: None,
        face_vertex_counts: None,
    })
}

pub struct MeshData {
    pub left_handed: bool,
    pub points_data: Option<Vec<f32>>,
    pub points_interpolation: Option<Interpolation>,
    pub normals_data: Option<Vec<f32>>,
    pub normals_interpolation: Option<Interpolation>,
    pub uvs_data: Option<Vec<f32>>,
    pub uvs_interpolation: Option<Interpolation>,
    pub face_vertex_indices: Option<Vec<i32>>,
    pub face_vertex_counts: Option<Vec<i32>>,
}
impl MeshData {
    pub fn set_left_handed(&mut self, left_handed: bool) {
        self.left_handed = left_handed;
    }

    pub fn set_points(&mut self, data: &[f32], interpolation: u8) {
        self.points_data = Some(data.to_vec());
        self.points_interpolation = Some(interpolation.into());
    }

    pub fn set_normals(&mut self, data: &[f32], interpolation: u8) {
        self.normals_data = Some(data.to_vec());
        self.normals_interpolation = Some(interpolation.into());
    }

    pub fn set_uvs(&mut self, data: &[f32], interpolation: u8) {
        self.uvs_data = Some(data.to_vec());
        self.uvs_interpolation = Some(interpolation.into());
    }

    pub fn set_face_vertex_indices(&mut self, data: &[i32]) {
        self.face_vertex_indices = Some(data.to_vec());
    }

    pub fn set_face_vertex_counts(&mut self, data: &[i32]) {
        self.face_vertex_counts = Some(data.to_vec());
    }
}

pub struct BridgeSender {
    sender: Sender<BridgeData>,
}
impl BridgeSender {
    pub fn new(sender: Sender<BridgeData>) -> Self {
        Self { sender }
    }

    pub fn message(&self, s: String) {
        let data = BridgeData::Message(s);
        self.sender.send(data).unwrap();
    }

    pub fn time_code_range(&self, start: f64, end: f64) {
        let data = BridgeData::TimeCodeRange(start, end);
        self.sender.send(data).unwrap();
    }

    pub fn create_mesh(&self, path: String) {
        let data = BridgeData::CreateMesh(UsdSdfPath(path));
        self.sender.send(data).unwrap();
    }

    pub fn transform_matrix(&self, path: String, matrix: &[f64]) {
        // row-major to column-major
        let mut data = [0.0_f32; 16];
        for r in 0..4 {
            for c in 0..4 {
                data[r * 4 + c] = matrix[c * 4 + r] as f32;
            }
        }
        let data = BridgeData::TransformMatrix(UsdSdfPath(path), data);
        self.sender.send(data).unwrap();
    }

    pub fn mesh_data(&self, path: String, data: Box<MeshData>) {
        let data = BridgeData::MeshData(UsdSdfPath(path), data.into());
        self.sender.send(data).unwrap();
    }

    pub fn destroy_mesh(&self, path: String) {
        let data = BridgeData::DestroyMesh(UsdSdfPath(path));
        self.sender.send(data).unwrap();
    }
}

pub struct BridgeSendEndNotifier {
    sender: Option<oneshot::Sender<()>>,
}
impl BridgeSendEndNotifier {
    pub fn new() -> (Box<Self>, oneshot::Receiver<()>) {
        let (sender, receiver) = oneshot::channel();
        (
            Box::new(Self {
                sender: Some(sender),
            }),
            receiver,
        )
    }

    pub fn notify(&mut self) {
        self.sender.take().unwrap().send(()).unwrap();
    }
}
