use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    let target_os = std::env::var("CARGO_CFG_TARGET_OS");
    let target_vendor = std::env::var("CARGO_CFG_TARGET_VENDOR");

    if target_os.as_deref() != Ok("windows")
        && target_os.as_deref() != Ok("android")
        && target_vendor.as_deref() != Ok("apple")
    {
        let mut modules = vec![
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
        ];
        let mut qbuild = qt_build_utils::QtBuild::new(modules.clone()).unwrap();

        let major = qbuild.version().major;
        if major != 5 && major != 6 {
            panic!("Unsupported Qt version: {major}");
        }
        println!("cargo::rustc-check-cfg=cfg(qtver, values(\"5\", \"6\"))");
        println!("cargo::rustc-cfg=qtver=\"{major}\"");

        let sources = [
            "src/runtime/qt",
            "src/common",
            "src/widgets/widget",
            "src/platform/image",
            "src/platform/monitor",
            "src/platform/timer",
            "src/dialogs/msgbox",
            "src/dialogs/filebox",
            "src/menu/ffi",
            "src/tool_bar/ffi",
            "src/widgets/window",
            "src/widgets/button",
            "src/widgets/menu_button",
            "src/widgets/canvas",
            "src/widgets/edit",
            "src/widgets/up_down_edit",
            "src/widgets/label",
            "src/widgets/progress",
            "src/widgets/combo_box",
            "src/widgets/list_box",
            "src/widgets/scroll_bar",
            "src/widgets/scroll_view",
            "src/widgets/tab_view",
            "src/widgets/group_box",
            #[cfg(feature = "media")]
            "src/widgets/media",
            #[cfg(feature = "webview")]
            "src/widgets/webview",
            #[cfg(feature = "wgpu")]
            "src/widgets/wgpu",
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
        qbuild.cargo_link_libraries(&mut build);
        build.compile("winio");
    }
}
