#![cfg_attr(feature = "once_cell_try", feature(once_cell_try))]
#![cfg_attr(feature = "lazy_cell", feature(lazy_cell))]

mod runtime;
mod ui;

pub use runtime::block_on;
#[cfg(windows)]
pub(crate) use runtime::{wait, window_proc};
pub use ui::*;
