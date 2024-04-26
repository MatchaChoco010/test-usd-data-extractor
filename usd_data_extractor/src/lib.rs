use std::path::Path;
use std::sync::mpsc::Receiver;

mod bridge;

pub enum BridgeData {
    Message(String),
    TimeCodeRange(f64, f64),
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
