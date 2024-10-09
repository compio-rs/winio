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
        let qbuild =
            qt_build_utils::QtBuild::new(vec!["Core".into(), "Gui".into(), "Widgets".into()])
                .unwrap();
        let qt_ver = qbuild.version();
        assert_eq!(qt_ver.major, 6);

        let sources = [
            "src/runtime/qt",
            "src/ui/qt/widget",
            "src/ui/qt/msgbox",
            "src/ui/qt/filebox",
            "src/ui/qt/window",
            "src/ui/qt/button",
            "src/ui/qt/canvas",
            "src/ui/qt/edit",
        ];

        for s in sources {
            println!("cargo:rerun-if-changed={}.rs", s);
            println!("cargo:rerun-if-changed={}.cpp", s);
            println!("cargo:rerun-if-changed={}.hpp", s);
        }

        let inc = qbuild.include_paths();

        let mut build = cxx_build::bridges(sources.map(|s| format!("{}.rs", s)));
        build
            .std("c++20")
            .files(sources.map(|s| format!("{}.cpp", s)))
            .includes(inc);
        if std::env::var("PROFILE").as_deref() == Ok("release") {
            build.flag("-flto").compiler("clang++");
        }
        qbuild.cargo_link_libraries(&mut build);
        build.compile("winio");
    }
}
