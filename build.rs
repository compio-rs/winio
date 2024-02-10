fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=app.manifest");
    println!("cargo:rerun-if-changed=manifest.rc");
    embed_resource::compile("manifest.rc", embed_resource::NONE);
}
