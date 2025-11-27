//! GTK backend for winio.

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
pub enum Error {
    /// IO error.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    /// Bool error.
    #[error("Bool error: {0}")]
    Bool(#[from] gtk4::glib::BoolError),
    /// Glib error.
    #[error("Glib error: {0}")]
    Glib(#[from] gtk4::glib::Error),
    /// Cairo error.
    #[error("Cairo error: {0}")]
    Cairo(#[from] gtk4::cairo::Error),
    /// Index error.
    #[error("Index error: {0}")]
    Index(usize),
    /// Null pointer returned.
    #[error("Null pointer returned")]
    NullPointer,
    /// Cast failed.
    #[error("Cast failed")]
    CastFailed,
    /// Color theme is not available.
    #[error("Color theme is not available")]
    NoColorTheme,
}

/// Result type for GTK.
pub type Result<T, E = Error> = std::result::Result<T, E>;
