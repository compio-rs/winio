fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    let target_os = std::env::var("CARGO_CFG_TARGET_OS");
    if target_os.as_deref() == Ok("windows") {
        println!("cargo:rerun-if-changed=app.manifest");
        println!("cargo:rerun-if-changed=manifest.rc");
        embed_resource::compile("manifest.rc", embed_resource::NONE)
            .manifest_required()
            .unwrap();
    }

    #[cfg(feature = "qt")]
    if target_os.as_deref() != Ok("windows") && target_os.as_deref() != Ok("macos") {
        let qbuild =
            qt_build_utils::QtBuild::new(vec!["Core".into(), "Gui".into(), "Widgets".into()])
                .unwrap();

        let major = qbuild.version().major;
        if major != 5 && major != 6 {
            panic!("Unsupported Qt version: {major}");
        }
        println!("cargo::rustc-check-cfg=cfg(qtver, values(\"5\", \"6\"))");
        println!("cargo::rustc-cfg=qtver=\"{major}\"");

        let sources = [
            "src/runtime/qt",
            "src/ui/qt/common",
            "src/ui/qt/widget",
            "src/ui/qt/monitor",
            "src/ui/qt/msgbox",
            "src/ui/qt/filebox",
            "src/ui/qt/window",
            "src/ui/qt/button",
            "src/ui/qt/canvas",
            "src/ui/qt/edit",
            "src/ui/qt/label",
            "src/ui/qt/progress",
            "src/ui/qt/combo_box",
            "src/ui/qt/list_box",
        ];

        for s in sources {
            println!("cargo:rerun-if-changed={s}.rs");
            println!("cargo:rerun-if-changed={s}.cpp");
            println!("cargo:rerun-if-changed={s}.hpp");
        }

        let inc = qbuild.include_paths();

        let mut build = cxx_build::bridges(sources.map(|s| format!("{s}.rs")));
        build
            .std("c++17")
            .files(sources.iter().filter_map(|s| {
                use std::path::PathBuf;
                let path = PathBuf::from(format!("{s}.cpp"));
                if path.exists() { Some(path) } else { None }
            }))
            .includes(inc)
            .cpp(true);
        qbuild.cargo_link_libraries(&mut build);
        build.compile("winio");
    }
}
