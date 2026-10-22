//! Qt backend for winio.

#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg(not(any(windows, target_vendor = "apple", target_os = "android")))]

pub(crate) use winio_pollable::GlobalRuntime;

mod runtime;
pub use runtime::*;

mod common;

mod widgets;
pub use widgets::*;

mod dialogs;
pub use dialogs::*;

mod platform;
pub use platform::*;

mod menu;
pub use menu::*;

mod tool_bar;
pub use tool_bar::*;

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// IO error.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    /// C++ exception.
    #[error("C++ exception: {0}")]
    Cxx(#[from] cxx::Exception),
    /// Index error.
    #[error("Index error: {0}")]
    Index(usize),
    /// UTF8 error.
    #[error("Invalid UTF-8 string")]
    InvalidUtf8(#[from] std::str::Utf8Error),
    /// Channel recv error.
    #[error("Channel recv error: {0}")]
    ChannelRecv(#[from] local_sync::oneshot::error::RecvError),
    /// Media player error.
    #[cfg(feature = "media")]
    #[error("Media player error: {0:?}")]
    Media(#[from] widgets::QMediaPlayerError),
    /// Time component range error.
    #[cfg(feature = "webview")]
    #[error("Time component range error: {0}")]
    TimeRange(#[from] time::error::ComponentRange),
    /// Feature not supported.
    #[error("Feature not supported")]
    NotSupported,
    /// A menu bar already exists.
    #[error("A menu bar already exists")]
    MenuBarExists,
    /// A tool bar already exists.
    #[error("A tool bar already exists")]
    ToolBarExists,
}

/// Result type for Qt.
pub type Result<T> = std::result::Result<T, Error>;
