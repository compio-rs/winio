#![feature(thread_local)]
#![feature(once_cell_try, lazy_cell)]

pub mod msgbox;
mod runtime;
pub mod window;

use std::io;

pub(crate) use runtime::window_proc;
pub use runtime::{block_on, spawn, wait};
use windows_sys::Win32::Foundation::BOOL;

pub(crate) fn syscall_bool(res: BOOL) -> io::Result<()> {
    if res != 0 {
        Ok(())
    } else {
        Err(io::Error::last_os_error())
    }
}
