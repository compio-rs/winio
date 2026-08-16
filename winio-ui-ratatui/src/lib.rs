pub use std::io::{Error, Result};

pub(crate) fn not_impl() -> ! {
    unimplemented!("stub implementation")
}

pub(crate) fn not_impl_fut<T>() -> std::future::Ready<T> {
    not_impl()
}

mod runtime;
pub use runtime::*;

mod widgets;
pub use widgets::*;

mod dialogs;
pub use dialogs::*;

mod platform;
pub use platform::*;

#[cfg(feature = "compio-compat")]
mod compat;
#[cfg(feature = "compio-compat")]
pub use compat::*;
