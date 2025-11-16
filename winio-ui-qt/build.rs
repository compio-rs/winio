use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    let target_os = std::env::var("CARGO_CFG_TARGET_OS");

    if target_os.as_deref() != Ok("windows") && target_os.as_deref() != Ok("macos") {
        let qbuild = qt_build_utils::QtBuild::new(vec![
            "Core".into(),
            "Gui".into(),
            "Widgets".into(),
            #[cfg(feature = "media")]
            "Multimedia".into(),
            #[cfg(feature = "media")]
            "MultimediaWidgets".into(),
            #[cfg(feature = "webview")]
            "WebEngineCore".into(),
            #[cfg(feature = "webview")]
            "WebEngineWidgets".into(),
            #[cfg(feature = "opengl")]
            "OpenGLWidgets".into(),
        ])
        .unwrap();

        let major = qbuild.version().major;
        if major != 5 && major != 6 {
            panic!("Unsupported Qt version: {major}");
        }
        println!("cargo::rustc-check-cfg=cfg(qtver, values(\"5\", \"6\"))");
        println!("cargo::rustc-cfg=qtver=\"{major}\"");

        let sources = [
            "src/runtime/qt",
            "src/ui/common",
            "src/ui/widget",
            "src/ui/monitor",
            "src/ui/msgbox",
            "src/ui/filebox",
            "src/ui/window",
            "src/ui/button",
            "src/ui/canvas",
            "src/ui/edit",
            "src/ui/label",
            "src/ui/progress",
            "src/ui/combo_box",
            "src/ui/list_box",
            "src/ui/scroll_bar",
            "src/ui/scroll_view",
            "src/ui/tab_view",
            #[cfg(feature = "media")]
            "src/ui/media",
            #[cfg(feature = "webview")]
            "src/ui/webview",
        ];

        for s in sources {
            println!("cargo:rerun-if-changed={s}.rs");
            println!("cargo:rerun-if-changed={s}.hpp");
            if PathBuf::from(format!("{s}.cpp")).exists() {
                println!("cargo:rerun-if-changed={s}.cpp");
            }
        }

        let inc = qbuild.include_paths();

        let mut build = cxx_build::bridges(sources.map(|s| format!("{s}.rs")));
        build
            .std("c++17")
            .files(sources.iter().filter_map(|s| {
                let path = PathBuf::from(format!("{s}.cpp"));
                if path.exists() { Some(path) } else { None }
            }))
            .includes(inc)
            .cpp(true);
        if cfg!(feature = "opengl") {
            build.define("WINIO_UI_QT_OPENGL", None);
        }
        qbuild.cargo_link_libraries(&mut build);
        build.compile("winio");
    }
}
