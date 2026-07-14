/// A stub error type.
#[derive(Debug)]
pub struct Error(std::convert::Infallible);

impl std::fmt::Display for Error {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        not_impl()
    }
}

impl std::error::Error for Error {}

/// A stub result type.
pub type Result<T, E = Error> = std::result::Result<T, E>;

impl From<std::io::Error> for Error {
    fn from(_err: std::io::Error) -> Self {
        not_impl()
    }
}

pub fn not_impl() -> ! {
    unimplemented!("stub implementation")
}

mod runtime;
pub use runtime::*;

mod widgets;
pub use widgets::*;

mod dialogs;
pub use dialogs::*;

#[cfg(feature = "compio-compat")]
mod compat;
#[cfg(feature = "compio-compat")]
pub use compat::*;
