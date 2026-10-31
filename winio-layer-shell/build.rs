use std::{
    env, fs,
    path::{Path, PathBuf},
};

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src/qt.cpp");
    println!("cargo:rerun-if-changed=src/qt.hpp");
    println!("cargo:rerun-if-changed=src/qt.rs");
    println!("cargo:rerun-if-env-changed=LAYERSHELLQT_INCLUDE_DIR");
    println!("cargo:rerun-if-env-changed=LAYERSHELLQT_LIB_DIR");

    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    let target_vendor = env::var("CARGO_CFG_TARGET_VENDOR").unwrap_or_default();
    if target_os == "windows" || target_os == "android" || target_vendor == "apple" {
        return;
    }
    if env::var_os("CARGO_FEATURE_QT").is_none() {
        return;
    }

    let qt = qt_build_utils::QtBuild::new(vec!["Core".into(), "Gui".into(), "Widgets".into()])
        .expect("failed to find Qt; install the Qt development packages");

    let include_dir = find_include_dir(&qt);
    let lib_dir = find_lib_dir(&include_dir);

    let mut build = cxx_build::bridge("src/qt.rs");
    build
        .cpp(true)
        .std("c++17")
        .warnings(true)
        // Version 6.8 of the LayerShellQt headers put a deprecation attribute
        // in front of an enum, which GCC ignores with a warning.
        .flag_if_supported("-Wno-attributes")
        .file("src/qt.cpp")
        .include(&include_dir);
    for include in qt.include_paths() {
        build.include(include);
    }

    println!("cargo:rustc-link-search=native={}", lib_dir.display());
    println!("cargo:rustc-link-lib=LayerShellQtInterface");
    qt.cargo_link_libraries(&mut build);
    build.compile("winio-layer-shell-qt");
}

fn find_include_dir(qt: &qt_build_utils::QtBuild) -> PathBuf {
    if let Some(dir) = env::var_os("LAYERSHELLQT_INCLUDE_DIR").map(PathBuf::from) {
        if dir.join("LayerShellQt").is_dir() {
            return dir;
        }
        if dir.file_name().is_some_and(|name| name == "LayerShellQt")
            && let Some(parent) = dir.parent()
        {
            return parent.to_owned();
        }
        panic!(
            "LAYERSHELLQT_INCLUDE_DIR must contain the LayerShellQt directory: {}",
            dir.display()
        );
    }

    for include in qt.include_paths() {
        for dir in include.ancestors() {
            if dir.join("LayerShellQt").is_dir() {
                return dir.to_owned();
            }
        }
    }
    for dir in ["/usr/include", "/usr/local/include"] {
        let dir = Path::new(dir);
        if dir.join("LayerShellQt").is_dir() {
            return dir.to_owned();
        }
    }

    panic!(
        "LayerShellQt development files not found. Install the layer-shell-qt development \
         package, or set LAYERSHELLQT_INCLUDE_DIR and LAYERSHELLQT_LIB_DIR."
    );
}

fn find_lib_dir(include_dir: &Path) -> PathBuf {
    if let Some(dir) = env::var_os("LAYERSHELLQT_LIB_DIR").map(PathBuf::from) {
        if has_library(&dir) {
            return dir;
        }
        panic!(
            "LAYERSHELLQT_LIB_DIR does not contain libLayerShellQtInterface: {}",
            dir.display()
        );
    }

    let mut candidates = Vec::new();
    if let Some(prefix) = include_dir.parent() {
        candidates.push(prefix.join("lib"));
        candidates.push(prefix.join("lib64"));
        candidates.push(prefix.join("lib/x86_64-linux-gnu"));
        candidates.push(prefix.join("lib/aarch64-linux-gnu"));
    }
    candidates.extend(
        [
            "/usr/lib",
            "/usr/lib64",
            "/usr/local/lib",
            "/usr/lib/x86_64-linux-gnu",
            "/usr/lib/aarch64-linux-gnu",
        ]
        .map(PathBuf::from),
    );

    for dir in candidates {
        if has_library(&dir) {
            return dir;
        }
    }

    panic!(
        "libLayerShellQtInterface not found. Install the layer-shell-qt development package, or \
         set LAYERSHELLQT_INCLUDE_DIR and LAYERSHELLQT_LIB_DIR."
    );
}

fn has_library(dir: &Path) -> bool {
    fs::read_dir(dir).is_ok_and(|entries| {
        entries.filter_map(|entry| entry.ok()).any(|entry| {
            let name = entry.file_name();
            let name = name.to_string_lossy();
            name.starts_with("libLayerShellQtInterface.so")
                || name.starts_with("libLayerShellQtInterface.dylib")
        })
    })
}
