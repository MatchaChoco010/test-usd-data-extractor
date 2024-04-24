use std::path::Path;

mod bridge;

pub struct UsdDataExtractor {
    inner: cxx::UniquePtr<bridge::ffi::BridgeUsdDataExtractor>,
}
impl UsdDataExtractor {
    pub fn new(path: impl AsRef<Path>) -> Self {
        let inner = bridge::ffi::new_usd_data_extractor(path.as_ref().to_str().unwrap());
        Self { inner }
    }
}
