#[cfg(not(target_os = "android"))]
fn main() -> main::Result<()> {
    use main::MainModel;
    use winio::prelude::*;

    tracing_subscriber::fmt()
        .with_max_level(compio_log::Level::INFO)
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
