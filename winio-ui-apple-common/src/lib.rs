use objc2::{exception::Exception, rc::Retained};
use objc2_foundation::NSError;

/// Error type for Apple backends.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Objective-C exception: {0:?}")]
    ObjC(Option<Retained<Exception>>),
    #[error("NSError: {0:?}")]
    NS(Option<Retained<NSError>>),
    #[error("Channel recv error: {0}")]
    ChannelRecv(#[from] local_sync::oneshot::error::RecvError),
    #[error("Null pointer returned")]
    NullPointer,
    #[error("Called from non-main thread")]
    NotMainThread,
    #[error("Feature not supported")]
    NotSupported,
}

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

pub type Result<T, E = Error> = std::result::Result<T, E>;

pub fn catch<F, R>(f: F) -> Result<R>
where
    F: FnOnce() -> R,
{
    match objc2::exception::catch(std::panic::AssertUnwindSafe(f)) {
        Ok(v) => Ok(v),
        Err(exc) => Err(Error::from(exc)),
    }
}

mod drawing;
pub use drawing::*;

mod string;
pub use string::*;
