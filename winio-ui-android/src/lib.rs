//! Android backend for winio.

#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg(target_os = "android")]

#[cfg(feature = "compio-compat")]
mod compat;
#[cfg(feature = "compio-compat")]
pub use compat::*;

mod convert;
pub(crate) use convert::*;

mod runtime;
pub use runtime::*;

mod ui;
pub use ui::*;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// JNI error.
    #[error("JNI error: {0}")]
    Jni(#[from] jni::errors::Error),
    /// IO error.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    /// No application available.
    #[error("No application available")]
    NoApp,
    /// Feature not supported.
    #[error("Feature not supported")]
    NotSupported,
}

pub type Result<T> = std::result::Result<T, Error>;
