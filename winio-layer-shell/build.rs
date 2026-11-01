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
    // Layer shell is Wayland-only (not Windows/Apple/Android).
    let is_linux = target_os != "windows" && target_os != "android" && target_vendor != "apple";
    if !is_linux {
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
        build.include(&include);
        // The platform-native-interface header (qpa/qplatformnativeinterface.h)
        // is reachable as <QtGui/qpa/...> only from a specific parent:
        //   Ubuntu:  .../qt6                (contains QtGui/qpa/...)
        //   Nix:     .../QtGui/<version>    (contains QtGui/qpa/...)
        if include.ends_with("QtGui") {
            if let Some(parent) = include.parent() {
                build.include(parent);
            }
            if let Ok(entries) = fs::read_dir(&include) {
                for entry in entries.flatten() {
                    let versioned = entry.path();
                    if versioned
                        .join("QtGui/qpa/qplatformnativeinterface.h")
                        .is_file()
                    {
                        build.include(&versioned);
                    }
                }
            }
        }
    }

    println!("cargo:rustc-link-search=native={}", lib_dir.display());
    println!("cargo:rustc-link-lib=LayerShellQtInterface");
    qt.cargo_link_libraries(&mut build);

    // Probe the layer-shell protocol via libwayland-client.
    let (wl_inc, wl_lib) = find_wayland_client();
    build.include(&wl_inc);
    println!("cargo:rustc-link-search=native={}", wl_lib.display());
    println!("cargo:rustc-link-lib=wayland-client");

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
    dir.join("libLayerShellQtInterface.so").is_file()
}

fn has_wayland_library(dir: &Path) -> bool {
    fs::read_dir(dir).is_ok_and(|entries| {
        entries.filter_map(|entry| entry.ok()).any(|entry| {
            entry
                .file_name()
                .to_string_lossy()
                .starts_with("libwayland-client.so")
        })
    })
}

fn find_wayland_client() -> (PathBuf, PathBuf) {
    if let (Some(inc), Some(lib)) = (
        env::var_os("WAYLAND_CLIENT_INCLUDE_DIR").map(PathBuf::from),
        env::var_os("WAYLAND_CLIENT_LIB_DIR").map(PathBuf::from),
    ) {
        if inc.join("wayland-client-core.h").is_file() && has_wayland_library(&lib) {
            return (inc, lib);
        }
        panic!(
            "WAYLAND_CLIENT_INCLUDE_DIR/LIB_DIR invalid: {} / {}",
            inc.display(),
            lib.display()
        );
    }

    if let Ok(out) = std::process::Command::new("pkg-config")
        .args(["--cflags-only-I", "--libs-only-L", "wayland-client"])
        .output()
        && out.status.success()
    {
        let mut inc = None;
        let mut lib = None;
        for token in String::from_utf8_lossy(&out.stdout).split_whitespace() {
            if let Some(path) = token.strip_prefix("-I") {
                inc = Some(PathBuf::from(path));
            } else if let Some(path) = token.strip_prefix("-L") {
                lib = Some(PathBuf::from(path));
            }
        }
        if let (Some(inc), Some(lib)) = (inc, lib) {
            return (inc, lib);
        }
    }

    for inc in ["/usr/include", "/usr/local/include"] {
        let inc = Path::new(inc);
        if inc.join("wayland-client-core.h").is_file() {
            for lib in [
                "/usr/lib",
                "/usr/lib64",
                "/usr/local/lib",
                "/usr/lib/x86_64-linux-gnu",
            ]
            .map(PathBuf::from)
            {
                if has_wayland_library(&lib) {
                    return (inc.to_owned(), lib);
                }
            }
        }
    }

    panic!(
        "libwayland-client not found. Install the Wayland development package, or set \
         WAYLAND_CLIENT_INCLUDE_DIR and WAYLAND_CLIENT_LIB_DIR."
    );
}
