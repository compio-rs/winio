#[cfg(feature = "hello")]
#[path = "../../winio/examples/hello.rs"]
mod hello;

#[cfg(feature = "widgets")]
#[path = "../../winio/examples/widgets.rs"]
mod widgets;

#[cfg(feature = "hello")]
use hello::*;

#[cfg(feature = "widgets")]
use widgets::*;

use winio::prelude::*;

#[unsafe(no_mangle)]
fn android_main(app: AndroidApp) {
    #[cfg(debug_assertions)]
    unsafe {
        std::env::set_var("RUST_BACKTRACE", "full");
    }

    App::new("rs.compio.winio.widgets")
        .set_android_app(app)
        .run::<MainModel>(());
}
