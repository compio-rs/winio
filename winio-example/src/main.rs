#[cfg(not(target_os = "android"))]
fn main() -> main::Result<()> {
    use main::MainModel;
    use winio::prelude::*;

    tracing_subscriber::fmt()
        .with_max_level(compio_log::Level::INFO)
        .init();

    grow_stack(|| {
        App::builder()
            .name("rs.compio.winio.widgets")
            .build()?
            .block_on(MainModel::run_until_event(()))
    })
}

#[cfg(target_os = "android")]
fn main() {
    unreachable!("Android entry point is `android_main` in `android.rs`")
}

#[cfg(windows)]
fn grow_stack<R, F: FnOnce() -> R>(f: F) -> R {
    stacker::grow(8 * 1024 * 1024, f)
}

#[cfg(not(windows))]
#[allow(unused)]
fn grow_stack<R, F: FnOnce() -> R>(f: F) -> R {
    f()
}
