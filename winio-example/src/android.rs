use android_activity::AndroidApp;
use compio_log::metadata::LevelFilter;
use tracing_subscriber::prelude::*;
use winio::prelude::*;

use crate::MainModel;

#[unsafe(no_mangle)]
fn android_main(app: AndroidApp) {
    unsafe {
        std::env::set_var("RUST_BACKTRACE", "1");
    }

    tracing_subscriber::registry()
        .with(tracing_android_trace::AndroidTraceLayer::new())
        .with(
            tracing_subscriber::fmt::layer()
                .with_ansi(false)
                .with_filter(LevelFilter::DEBUG),
        )
        .try_init()
        .ok();

    let app = App::builder()
        .android_app(app)
        .build()
        .expect("cannot create app");
    app.spawn(|| async {
        if let Err(e) = MainModel::run_until_event(()).await {
            compio_log::error!("App error: {e:?}");
        }
    })
}
