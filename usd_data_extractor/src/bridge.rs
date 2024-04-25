use std::sync::mpsc::Sender;

use crate::BridgeSenderData;

#[cxx::bridge]
pub mod ffi {
    extern "Rust" {
        pub type BridgeSender;
        fn send_string(&self, s: String);
    }
    unsafe extern "C++" {
        // include!("usd_data_extractor/include/include.h");
        include!("usd_data_extractor/cpp/usdDataExtractor.h");

        type BridgeUsdDataExtractor;
        fn extract(&self);

        fn new_usd_data_extractor(
            sender: Box<BridgeSender>,
            open_path: &str,
        ) -> UniquePtr<BridgeUsdDataExtractor>;
    }
}

pub struct BridgeSender {
    sender: Sender<BridgeSenderData>,
}
impl BridgeSender {
    pub fn new(sender: Sender<BridgeSenderData>) -> Self {
        Self { sender }
    }

    pub fn send_string(&self, s: String) {
        let data = BridgeSenderData::String(s);
        self.sender.send(data).unwrap();
    }
}
