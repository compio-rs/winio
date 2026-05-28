#[cfg(not(target_os = "android"))]
fn main() -> winio_example::Result<()> {
    use winio::prelude::*;
    use winio_example::MainModel;

    tracing_subscriber::fmt()
        .with_max_level(compio_log::Level::DEBUG)
        .init();

    App::builder()
        .name("rs.compio.winio.widgets")
        .build()?
        .block_on(MainModel::run_until_event(()))
}

#[cfg(target_os = "android")]
fn main() {
    unreachable!("Android entry point is `android_main` in `android.rs`")
}
