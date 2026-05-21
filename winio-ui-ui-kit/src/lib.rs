//! UIKit backend for winio.

#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg(target_os = "ios")]

mod runtime;
use objc2::{exception::Exception, rc::Retained};
use objc2_foundation::NSError;
pub use runtime::*;

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
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
    /// Called from non-main thread.
    #[error("Called from non-main thread")]
    NotMainThread,
}

// SAFETY: NSException & NSError are thread-safe.
unsafe impl Send for Error {}
unsafe impl Sync for Error {}

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

#[cfg(feature = "compio-compat")]
pub use compio::compat::FuturesAdapter as CompioAdapter;
