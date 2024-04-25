use std::path::Path;
use std::sync::mpsc::Receiver;

mod bridge;

pub enum BridgeSenderData {
    String(String),
}

pub struct UsdDataExtractor {
    inner: cxx::UniquePtr<bridge::ffi::BridgeUsdDataExtractor>,
    rx: Receiver<BridgeSenderData>,
}
impl UsdDataExtractor {
    pub fn new(path: impl AsRef<Path>) -> Self {
        let (tx, rx) = std::sync::mpsc::channel();
        let sender = Box::new(bridge::BridgeSender::new(tx));
        let inner = bridge::ffi::new_usd_data_extractor(sender, path.as_ref().to_str().unwrap());
        Self { inner, rx }
    }

    pub fn show_data(&mut self) {
        self.inner.extract();

        while let Ok(data) = self.rx.recv() {
            match data {
                BridgeSenderData::String(s) => {
                    println!("{}", s);
                }
            }
        }
    }
}
