#![feature(thread_local)]
#![feature(once_cell_try, lazy_cell)]

mod runtime;
pub mod ui;

use std::io;

pub(crate) use runtime::window_proc;
pub use runtime::{block_on, spawn, wait};
use windows_sys::Win32::Foundation::BOOL;

pub(crate) fn syscall_bool(res: BOOL) -> io::Result<BOOL> {
    if res != 0 {
        Ok(res)
    } else {
        Err(io::Error::last_os_error())
    }
}
