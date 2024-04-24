fn main() {
    cxx_build::bridge("src/bridge.rs")
        .cpp(true)
        .debug(false)
        .define("NOMINMAX", None)
        .flag_if_supported("-std=c++17")
        .flag_if_supported("/utf-8")
        .file("cpp/usdDataExtractor.cpp")
        .compile("usd-data-extractor-cpp");

    println!("cargo:rerun-if-changed=src/*.rs");
    println!("cargo:rerun-if-changed=cpp/*.cpp");
    println!("cargo:rerun-if-changed=include/include.h");
}
