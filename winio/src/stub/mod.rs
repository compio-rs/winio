/// A stub error type.
#[derive(Debug)]
pub struct Error(std::convert::Infallible);

impl Display for Error {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        not_impl()
    }
}

impl std::error::Error for Error {}

/// A stub result type.
pub type Result<T, E = Error> = std::result::Result<T, E>;

pub fn not_impl() -> ! {
    unimplemented!("stub implementation")
}

mod runtime;
use std::fmt::Display;

pub use runtime::*;

mod ui;
pub use ui::*;
