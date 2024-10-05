fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    let target_os = std::env::var("CARGO_CFG_TARGET_OS");
    if target_os.as_deref() == Ok("windows") {
        println!("cargo:rerun-if-changed=app.manifest");
        println!("cargo:rerun-if-changed=manifest.rc");
        embed_resource::compile("manifest.rc", embed_resource::NONE);
    }

    #[cfg(feature = "qt")]
    if target_os.as_deref() != Ok("windows") && target_os.as_deref() != Ok("macos") {
        let build =
            qt_build_utils::QtBuild::new(vec!["Core".into(), "Gui".into(), "Widgets".into()])
                .unwrap();
        let qt_ver = build.version();
        assert_eq!(qt_ver.major, 6);

        println!("cargo:rerun-if-changed=src/runtime/qt.rs");
        println!("cargo:rerun-if-changed=src/runtime/qt.cpp");
        println!("cargo:rerun-if-changed=src/runtime/qt.hpp");
        println!("cargo:rerun-if-changed=src/ui/qt/widget.rs");
        println!("cargo:rerun-if-changed=src/ui/qt/widget.cpp");
        println!("cargo:rerun-if-changed=src/ui/qt/widget.hpp");

        let inc = build.include_paths();

        cxx_build::bridges(["src/runtime/qt.rs", "src/ui/qt/widget.rs"])
            .files(["src/runtime/qt.cpp", "src/ui/qt/widget.cpp"])
            .includes(inc)
            .compile("winio");
    }
}
