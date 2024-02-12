#![feature(thread_local)]
#![feature(once_cell_try, lazy_cell)]

pub mod fs;
mod ioext;
mod runtime;
pub mod ui;

use std::io;

pub(crate) use ioext::*;
pub(crate) use runtime::window_proc;
pub use runtime::{block_on, spawn, wait};
use windows_sys::Win32::Foundation::{BOOL, HANDLE, INVALID_HANDLE_VALUE};

pub(crate) fn syscall_bool(res: BOOL) -> io::Result<BOOL> {
    if res != 0 {
        Ok(res)
    } else {
        Err(io::Error::last_os_error())
    }
}

pub(crate) fn syscall_handle(res: HANDLE) -> io::Result<HANDLE> {
    if res != INVALID_HANDLE_VALUE {
        Ok(res)
    } else {
        Err(io::Error::last_os_error())
    }
}
