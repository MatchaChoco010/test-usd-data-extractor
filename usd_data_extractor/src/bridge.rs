use std::sync::mpsc::Sender;

use crate::BridgeData;

#[cxx::bridge]
pub mod ffi {
    extern "Rust" {
        type BridgeSender;
        fn message(self: &BridgeSender, s: String);
        fn time_code_range(self: &BridgeSender, start: f64, end: f64);

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
