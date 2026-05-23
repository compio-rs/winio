//! GTK backend for winio.

#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg(not(any(windows, target_vendor = "apple", target_os = "android")))]

pub(crate) use winio_pollable::GlobalRuntime;

mod runtime;
pub use runtime::*;

#[cfg(feature = "compio-compat")]
mod compat;
#[cfg(feature = "compio-compat")]
pub use compat::*;

mod ui;
pub use ui::*;

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
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
    /// Feature not supported.
    #[error("Feature not supported")]
    NotSupported,
}

/// Result type for GTK.
pub type Result<T, E = Error> = std::result::Result<T, E>;
