//! Qt backend for winio.

#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg(not(any(windows, target_os = "macos")))]

use winio_callback::Runnable;

pub(crate) struct GlobalRuntime;

impl Runnable for GlobalRuntime {
    #[inline]
    fn run() {
        RUNTIME.with(|runtime| runtime.run())
    }
}

scoped_tls::scoped_thread_local!(pub(crate) static RUNTIME: Runtime);

mod runtime;
pub use runtime::*;

mod ui;
pub use ui::*;

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
    /// Channel recv error.
    #[error("Channel recv error: {0}")]
    ChannelRecv(#[from] local_sync::oneshot::error::RecvError),
    /// Media player error.
    #[cfg(feature = "media")]
    #[error("Media player error: {0:?}")]
    Media(#[from] ui::QMediaPlayerError),
    /// Feature not supported.
    #[error("Feature not supported")]
    NotSupported,
}

/// Result type for Qt.
pub type Result<T, E = Error> = std::result::Result<T, E>;
