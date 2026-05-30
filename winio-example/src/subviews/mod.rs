mod scroll_view;
pub use scroll_view::*;

mod misc;
pub use misc::*;

cfg_if::cfg_if! {
    if #[cfg(feature = "compio-compat")] {
        mod fs;
        pub use fs::*;

        mod net;
        pub use net::*;

        mod gallery;
        pub use gallery::*;
    } else {
        pub use DummyPage as FsPage;
        pub use DummyPage as NetPage;
        pub use DummyPage as GalleryPage;
    }
}

cfg_if::cfg_if! {
    if #[cfg(feature = "plotters")] {
        mod plotters;
        pub use plotters::*;
    } else {
        pub use DummyPage as PlottersPage;
    }
}

cfg_if::cfg_if! {
    if #[cfg(all(feature = "media", feature = "compio-compat"))] {
        mod media;
        pub use media::*;
    } else {
        pub use DummyPage as MediaPage;
    }
}

cfg_if::cfg_if! {
    if #[cfg(feature = "webview")] {
        mod failable_webview;
        pub use failable_webview::*;

        mod webview;
        pub use webview::*;
    } else {
        pub use DummyPage as WebViewPage;
    }
}

cfg_if::cfg_if! {
    if #[cfg(all(feature = "webview", feature = "compio-compat"))] {
        mod markdown;
        pub use markdown::*;
    } else {
        pub use DummyPage as MarkdownPage;
    }
}

cfg_if::cfg_if! {
    if #[cfg(any(not(feature = "media"), not(feature = "webview"), not(feature = "plotters"), not(feature = "compio-compat")))] {
        mod dummy;
        pub use dummy::*;
    }
}
