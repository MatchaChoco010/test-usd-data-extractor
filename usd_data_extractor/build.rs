use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    let mut vcvars = vcvars::Vcvars::new();
    let vcvars_include = vcvars.get_cached("INCLUDE").unwrap();

    let mut vcvars = vcvars::Vcvars::new();
    let vcvars_lib = vcvars.get_cached("LIB").unwrap();

    let mut vcvars = vcvars::Vcvars::new();
    let vcvars_libpath = vcvars.get_cached("LIBPATH").unwrap();

    let mut vcvars = vcvars::Vcvars::new();
    let vcvars_path = vcvars.get_cached("PATH").unwrap();

    let mut vcvars = vcvars::Vcvars::new();
    let visual_studio_version = vcvars.get_cached("VisualStudioVersion").unwrap();

    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let usd_dir = PathBuf::from(&manifest_dir)
        .parent()
        .unwrap()
        .join("OpenUSD");
    let target_dir = PathBuf::from(&manifest_dir)
        .parent()
        .unwrap()
        .join("target");
    let usd_dir = usd_dir.to_str().unwrap();
    let usd_dst = PathBuf::from(&target_dir).join("OpenUSD");

    let pxr_lib_prefix = env::var("PXR_LIB_PREFIX").unwrap_or("usd".to_string());

    let output = Command::new("cmd")
        .env("INCLUDE", &*vcvars_include)
        .env("LIB", &*vcvars_lib)
        .env("LIBPATH", &*vcvars_libpath)
        .env("PATH", &*vcvars_path)
        .env("VisualStudioVersion", &*visual_studio_version)
        .env("PYTHONUTF8", "1")
        .args([
            "/c",
            "python",
            &format!("{usd_dir}/build_scripts/build_usd.py"),
            "--no-python",
            usd_dst.to_str().unwrap(),
            "--build-args",
            &format!("USD,\"-DPXR_LIB_PREFIX={pxr_lib_prefix}\""),
        ])
        .output()
        .expect("failed to execute build process");

    if !output.status.success() {
        println!("status: {}", output.status);
        println!("stdout: {}", String::from_utf8_lossy(&output.stdout));
        println!("stderr: {}", String::from_utf8_lossy(&output.stderr));
        panic!("failed to build OpenUSD");
    }

    // collect cpp files
    let mut cpp_files = vec![];
    for f in fs::read_dir("cpp").unwrap() {
        let f = f.unwrap();
        let f = f.file_name();
        let f = f.to_str().unwrap();
        if f.ends_with(".cpp") {
            cpp_files.push(format!("{manifest_dir}/cpp/{f}"));
        }
    }

    cxx_build::bridge("src/bridge.rs")
        .cpp(true)
        .debug(false)
        .warnings(false)
        .define("NOMINMAX", None)
        .flag("/W0")
        .flag_if_supported("/std:c++17")
        .flag_if_supported("-std=c++17")
        .flag_if_supported("/utf-8")
        .includes(env::split_paths(&*vcvars_include))
        .include(usd_dst.join("include"))
        .include(usd_dst.join("include/boost-1_78"))
        .files(cpp_files)
        .compile("usd-data-extractor-cpp");

    println!("cargo:rerun-if-changed={manifest_dir}/src/bridge.rs");
    println!("cargo:rerun-if-changed={manifest_dir}/cpp/");

    println!("cargo:rustc-link-search={}", usd_dst.join("lib").display());
    for f in fs::read_dir(usd_dst.join("lib")).unwrap() {
        let f = f.unwrap();
        let f = f.file_name();
        let f = f.to_str().unwrap();
        if f.ends_with(".lib") {
            let f = f.trim_end_matches(".lib");
            println!("cargo:rustc-link-lib=static={}", f);
        }
    }

    // copy dll to profile dir
    let profile = env::var("PROFILE").unwrap();
    let profile_dir = target_dir.join(profile);
    let usd_path = usd_dst.to_str().unwrap();
    for entry in glob::glob(&format!("{usd_path}/bin/*.dll")).unwrap() {
        if let Ok(path) = entry {
            fs::copy(&path, profile_dir.join(path.file_name().unwrap())).unwrap();
        }
    }

    // copy usd plugin dll, usda and json files to profile dir
    for entry in glob::glob(&format!("{usd_path}/lib/*.dll")).unwrap() {
        if let Ok(path) = entry {
            fs::copy(&path, profile_dir.join(path.file_name().unwrap())).unwrap();
        }
    }
    for entry in glob::glob(&format!("{usd_path}/lib/usd/**/*.json")).unwrap() {
        if let Ok(path) = entry {
            let relative_path = path.strip_prefix(&usd_dst.join("lib")).unwrap();
            if let Some(parent) = relative_path.parent() {
                fs::create_dir_all(profile_dir.join(parent)).unwrap();
            }
            fs::copy(&path, profile_dir.join(relative_path)).unwrap();
        }
    }
    for entry in glob::glob(&format!("{usd_path}/lib/usd/**/*.usda")).unwrap() {
        if let Ok(path) = entry {
            let relative_path = path.strip_prefix(&usd_dst.join("lib")).unwrap();
            if let Some(parent) = relative_path.parent() {
                fs::create_dir_all(profile_dir.join(parent)).unwrap();
            }
            fs::copy(&path, profile_dir.join(relative_path)).unwrap();
        }
    }
}
