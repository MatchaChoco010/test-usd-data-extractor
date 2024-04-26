use std::sync::mpsc::Sender;

use crate::{BridgeData, UsdSdfPath};

#[cxx::bridge]
pub mod ffi {
    extern "Rust" {
        type BridgeSender;
        fn message(self: &BridgeSender, s: String);
        fn time_code_range(self: &BridgeSender, start: f64, end: f64);
        fn transform_matrix(self: &BridgeSender, path: String, matrix: &[f64]);
        fn points(self: &BridgeSender, path: String, data: &[f32], interpolation: u8);
        fn normals(self: &BridgeSender, path: String, data: &[f32], interpolation: u8);
        fn uvs(self: &BridgeSender, path: String, data: &[f32], interpolation: u8);
        fn indices(self: &BridgeSender, path: String, data: &[i32]);

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

    pub fn points(&self, path: String, data: &[f32], interpolation: u8) {
        let data = BridgeData::Points(UsdSdfPath(path), data.to_vec(), interpolation.into());
        self.sender.send(data).unwrap();
    }

    pub fn normals(&self, path: String, data: &[f32], interpolation: u8) {
        let data = BridgeData::Normals(UsdSdfPath(path), data.to_vec(), interpolation.into());
        self.sender.send(data).unwrap();
    }

    pub fn uvs(&self, path: String, data: &[f32], interpolation: u8) {
        let data = BridgeData::Uvs(UsdSdfPath(path), data.to_vec(), interpolation.into());
        self.sender.send(data).unwrap();
    }

    pub fn indices(&self, path: String, data: &[i32]) {
        let data = data.iter().map(|&i| i as u32).collect::<Vec<_>>();
        let data = BridgeData::Indices(UsdSdfPath(path), data);
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
