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

        let sources = [
            "src/runtime/qt",
            "src/ui/qt/widget",
            "src/ui/qt/msgbox",
            "src/ui/qt/window",
            "src/ui/qt/button",
        ];

        for s in sources {
            println!("cargo:rerun-if-changed={}.rs", s);
            println!("cargo:rerun-if-changed={}.cpp", s);
            println!("cargo:rerun-if-changed={}.hpp", s);
        }

        let inc = build.include_paths();

        cxx_build::bridges(sources.map(|s| format!("{}.rs", s)))
            .files(sources.map(|s| format!("{}.cpp", s)))
            .includes(inc)
            .compile("winio");
    }
}
