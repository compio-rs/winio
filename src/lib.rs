#![cfg_attr(feature = "once_cell_try", feature(once_cell_try))]
#![cfg_attr(feature = "lazy_cell", feature(lazy_cell))]

mod runtime;
mod ui;

pub(crate) use runtime::window_proc;
pub use runtime::{block_on, wait};
pub use ui::*;
