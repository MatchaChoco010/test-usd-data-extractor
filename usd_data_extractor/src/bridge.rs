#[cxx::bridge]
pub mod ffi {
    unsafe extern "C++" {
        include!("usd_data_extractor/include/include.h");

        type BridgeUsdDataExtractor;

        fn new_usd_data_extractor() -> UniquePtr<BridgeUsdDataExtractor>;
    }
}
