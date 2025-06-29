//! Register eventfd to io-uring to make it pollable.

use std::{
    io,
    os::fd::{FromRawFd, OwnedFd, RawFd},
    task::Poll,
};

use compio::driver::syscall;

const IORING_REGISTER_EVENTFD: libc::c_uint = 4;

pub fn register_eventfd(ur: RawFd) -> io::Result<OwnedFd> {
    let ev_fd = syscall!(libc::eventfd(0, libc::EFD_CLOEXEC | libc::EFD_NONBLOCK))?;
    let ev = unsafe { OwnedFd::from_raw_fd(ev_fd) };
    syscall!(libc::syscall(
        libc::SYS_io_uring_register,
        ur,
        IORING_REGISTER_EVENTFD,
        std::ptr::addr_of!(ev_fd).cast::<libc::c_void>(),
        1i32
    ))?;
    Ok(ev)
}

pub fn eventfd_clear(fd: RawFd) -> io::Result<()> {
    let mut v: u64 = 0;
    match syscall!(break libc::read(fd, std::ptr::addr_of_mut!(v).cast(), 8)) {
        Poll::Pending => Ok(()),
        Poll::Ready(res) => res.map(|_| ()),
    }
}
