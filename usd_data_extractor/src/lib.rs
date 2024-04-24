mod bridge;

pub struct UsdDataExtractor {
    inner: cxx::UniquePtr<bridge::ffi::BridgeUsdDataExtractor>,
}
impl UsdDataExtractor {
    pub fn new() -> Self {
        let inner = bridge::ffi::new_usd_data_extractor();
        Self { inner }
    }
}
