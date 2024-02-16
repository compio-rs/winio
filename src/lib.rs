#![feature(thread_local)]
#![feature(once_cell_try, lazy_cell)]
#![feature(read_buf, core_io_borrowed_buf)]

pub mod fs;
pub mod http;
mod ioext;
pub mod net;
mod runtime;
pub mod time;
pub mod ui;

use std::io;

pub(crate) use ioext::*;
pub(crate) use runtime::window_proc;
pub use runtime::{block_on, spawn, wait};
use windows_sys::{
    core::HRESULT,
    Win32::Foundation::{HANDLE, INVALID_HANDLE_VALUE},
};

pub(crate) fn syscall_bool<T: Default + Eq>(res: T) -> io::Result<T> {
    if res != T::default() {
        Ok(res)
    } else {
        Err(io::Error::last_os_error())
    }
}

pub(crate) fn syscall_socket<T: Default + Eq>(res: T) -> io::Result<T> {
    if res == T::default() {
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

pub(crate) fn syscall_hresult(res: HRESULT) -> io::Result<()> {
    if res >= 0 {
        Ok(())
    } else {
        Err(io::Error::from_raw_os_error(res))
    }
}
