use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn build_windows(manifest_dir: &str, usd_dir: &str, usd_dst: &str, target_dir: PathBuf) {
    // get python path
    let python = which::which("python").unwrap();
    let python = python.to_str().unwrap();

    // build OpenUSD
    let output = Command::new("cmd")
        .env("PYTHONUTF8", "1")
        .arg("/c")
        .args([
            r#"cd /d C:\Program Files\Microsoft Visual Studio\2022\Community\Common7\Tools"#,
            "&&",
            "VsDevCmd.bat",
            "-arch=x64",
            "-host_arch=x64",
            "&&",
            python,
            &format!(r#"{usd_dir}\build_scripts\build_usd.py"#),
            "--no-python",
            usd_dst,
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

    // build CXX bridge
    let usd_dst = PathBuf::from(usd_dst);
    let mut vcvars = vcvars::Vcvars::new();
    let vcvars_include = vcvars.get_cached("INCLUDE").unwrap();
    cxx_build::bridge("src/bridge.rs")
        .cpp(true)
        .debug(false)
        .warnings(false)
        .define("NOMINMAX", None)
        .flag("/W0")
        .flag_if_supported("/std:c++20")
        .flag_if_supported("-std=c++20")
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

fn main() {
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
    let usd_dst_str = usd_dst.to_str().unwrap();

    if cfg!(target_os = "windows") {
        build_windows(&manifest_dir, &usd_dir, &usd_dst_str, target_dir);
    } else {
        panic!("Unsupported platform");
    }
}
