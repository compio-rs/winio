mod fs;
pub use fs::*;

mod net;
pub use net::*;

mod gallery;
pub use gallery::*;

mod scroll_view;
pub use scroll_view::*;

mod misc;
pub use misc::*;

cfg_if::cfg_if! {
    if #[cfg(feature = "plotters")] {
        mod plotters;
        pub use plotters::*;
    }
}

cfg_if::cfg_if! {
    if #[cfg(feature = "media")] {
        mod media;
        pub use media::*;
    }
}

cfg_if::cfg_if! {
    if #[cfg(feature = "webview")] {
        mod failable_webview;
        pub use failable_webview::*;

        mod webview;
        pub use webview::*;

        mod markdown;
        pub use markdown::*;
    }
}

cfg_if::cfg_if! {
    if #[cfg(any(not(feature = "media"), not(feature = "webview")))] {
        mod dummy;
        pub use dummy::*;
    }
}
