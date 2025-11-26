//! AppKit backend for winio.

#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg(target_os = "macos")]

use std::panic::AssertUnwindSafe;

use objc2::{exception::Exception, rc::Retained};
use objc2_foundation::NSError;
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

/// Error type for AppKit.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// IO error.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    /// Objective-C exception.
    #[error("Objective-C exception: {0:?}")]
    ObjC(Option<Retained<Exception>>),
    /// NSError.
    #[error("NSError: {0:?}")]
    NS(Option<Retained<NSError>>),
    /// Null pointer returned.
    #[error("Null pointer returned")]
    NullPointer,
    /// Called from non-main thread.
    #[error("Called from non-main thread")]
    NotMainThread,
    /// Feature not supported.
    #[error("Feature not supported")]
    NotSupported,
}

impl From<Retained<Exception>> for Error {
    fn from(exc: Retained<Exception>) -> Self {
        Error::ObjC(Some(exc))
    }
}

impl From<Option<Retained<Exception>>> for Error {
    fn from(exc: Option<Retained<Exception>>) -> Self {
        Error::ObjC(exc)
    }
}

impl From<Retained<NSError>> for Error {
    fn from(err: Retained<NSError>) -> Self {
        Error::NS(Some(err))
    }
}

impl From<Option<Retained<NSError>>> for Error {
    fn from(err: Option<Retained<NSError>>) -> Self {
        Error::NS(err)
    }
}

/// Result type for AppKit.
pub type Result<T, E = Error> = std::result::Result<T, E>;

pub(crate) fn catch<F, R>(f: F) -> Result<R>
where
    F: FnOnce() -> R,
{
    match objc2::exception::catch(AssertUnwindSafe(f)) {
        Ok(v) => Ok(v),
        Err(exc) => Err(Error::from(exc)),
    }
}
