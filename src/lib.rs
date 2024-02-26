#![feature(thread_local)]
#![feature(once_cell_try, lazy_cell)]
#![feature(read_buf, core_io_borrowed_buf)]

mod runtime;
pub mod ui;

pub(crate) use runtime::window_proc;
pub use runtime::{block_on, wait};
