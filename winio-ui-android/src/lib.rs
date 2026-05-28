//! Android backend for winio.

#![cfg_attr(docsrs, feature(doc_cfg, doc_auto_cfg))]
#![cfg(target_os = "android")]

#[cfg(feature = "compio-compat")]
pub use compio::compat::FuturesAdapter as CompioAdapter;

mod convert;
pub use convert::*;

mod runtime;
pub use runtime::*;

mod ui;
pub use ui::*;

pub type GlobalRef = jni::objects::Global<jni::objects::JObject<'static>>;

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
