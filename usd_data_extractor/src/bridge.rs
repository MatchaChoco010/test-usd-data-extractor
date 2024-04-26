use std::sync::mpsc::Sender;

use crate::BridgeSenderData;

#[cxx::bridge]
pub mod ffi {
    extern "Rust" {
        type BridgeSender;
        fn send_string(self: &BridgeSender, s: String);

        type BridgeSendEndNotifier;
        fn notify(self: &mut BridgeSendEndNotifier);
    }
    unsafe extern "C++" {
        include!("usd_data_extractor/cpp/usdDataExtractor.h");

        type BridgeUsdDataExtractor;
        fn extract(self: Pin<&mut BridgeUsdDataExtractor>, notifier: Box<BridgeSendEndNotifier>);
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
