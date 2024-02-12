use std::{
    future::Future,
    io,
    mem::ManuallyDrop,
    pin::Pin,
    task::{Context, Poll, Waker},
};

use compio_buf::{BufResult, SetBufInit};
use compio_log::debug;
use windows_sys::Win32::{
    Foundation::{
        GetLastError, ERROR_HANDLE_EOF, ERROR_IO_INCOMPLETE, ERROR_IO_PENDING, ERROR_NO_DATA,
        ERROR_PIPE_CONNECTED,
    },
    Networking::WinSock::LPWSAOVERLAPPED_COMPLETION_ROUTINE,
    System::IO::{LPOVERLAPPED_COMPLETION_ROUTINE, OVERLAPPED},
};

async fn with_overlapped_impl<C>(
    f: impl FnOnce(&mut OVERLAPPED, Option<C>) -> Poll<io::Result<usize>>,
    callback: C,
) -> io::Result<usize> {
    let mut fut = OverlappedFuture::new();
    match f(&mut fut.inner.base, Some(callback)) {
        Poll::Ready(res) => {
            debug!("operation ready: {:?}", res);
            res
        }
        Poll::Pending => {
            debug!("operation pending...");
            fut.await
        }
    }
}

pub(crate) async fn with_overlapped(
    f: impl FnOnce(&mut OVERLAPPED, LPOVERLAPPED_COMPLETION_ROUTINE) -> Poll<io::Result<usize>>,
) -> io::Result<usize> {
    with_overlapped_impl(f, overlapped_callback).await
}

pub(crate) async fn with_wsa_overlapped(
    f: impl FnOnce(&mut OVERLAPPED, LPWSAOVERLAPPED_COMPLETION_ROUTINE) -> Poll<io::Result<usize>>,
) -> io::Result<usize> {
    with_overlapped_impl(f, overlapped_wsa_callback).await
}

enum OverlappedFutureState {
    Active(Option<Waker>),
    Completed(io::Result<usize>),
    Fused,
}

#[repr(C)]
struct OverlappedInner {
    base: OVERLAPPED,
    state: OverlappedFutureState,
}

impl OverlappedInner {
    pub fn new() -> Self {
        Self {
            base: unsafe { std::mem::zeroed() },
            state: OverlappedFutureState::Active(None),
        }
    }
}

struct OverlappedFuture {
    inner: ManuallyDrop<Box<OverlappedInner>>,
}

impl OverlappedFuture {
    pub fn new() -> Self {
        Self {
            inner: ManuallyDrop::new(Box::new(OverlappedInner::new())),
        }
    }
}

impl Future for OverlappedFuture {
    type Output = io::Result<usize>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let state = std::mem::replace(&mut self.inner.state, OverlappedFutureState::Fused);
        match state {
            OverlappedFutureState::Active(_) => {
                self.inner.state = OverlappedFutureState::Active(Some(cx.waker().clone()));
                Poll::Pending
            }
            OverlappedFutureState::Completed(res) => Poll::Ready(res),
            OverlappedFutureState::Fused => unreachable!(),
        }
    }
}

impl Drop for OverlappedFuture {
    fn drop(&mut self) {
        if matches!(
            self.inner.state,
            OverlappedFutureState::Fused | OverlappedFutureState::Completed(_)
        ) {
            unsafe { ManuallyDrop::drop(&mut self.inner) }
        }
    }
}

unsafe extern "system" fn overlapped_callback(
    dwerrorcode: u32,
    dwnumberofbytestransfered: u32,
    lpoverlapped: *mut OVERLAPPED,
) {
    let ptr = lpoverlapped.cast::<OverlappedInner>();
    if let Some(fut) = ptr.as_mut() {
        let res = if dwerrorcode != 0 {
            if matches!(
                dwerrorcode,
                ERROR_IO_INCOMPLETE | ERROR_HANDLE_EOF | ERROR_NO_DATA
            ) {
                Ok(0)
            } else {
                Err(io::Error::from_raw_os_error(dwerrorcode as _))
            }
        } else {
            Ok(dwnumberofbytestransfered as usize)
        };
        let state = std::mem::replace(&mut fut.state, OverlappedFutureState::Completed(res));
        if let OverlappedFutureState::Active(Some(waker)) = state {
            waker.wake();
        }
    }
}

unsafe extern "system" fn overlapped_wsa_callback(
    dwerror: u32,
    cbtransferred: u32,
    lpoverlapped: *mut OVERLAPPED,
    _dwflags: u32,
) {
    overlapped_callback(dwerror, cbtransferred, lpoverlapped);
}

/// Trait to update the buffer length inside the [`BufResult`].
pub(crate) trait BufResultExt {
    /// Call [`SetBufInit::set_buf_init`] if the result is [`Ok`].
    fn map_advanced(self) -> Self;
}

impl<T: SetBufInit> BufResultExt for BufResult<usize, T> {
    fn map_advanced(self) -> Self {
        self.map_res(|res| (res, ()))
            .map_advanced()
            .map_res(|(res, _)| res)
    }
}

impl<T: SetBufInit, O> BufResultExt for BufResult<(usize, O), T> {
    fn map_advanced(self) -> Self {
        self.map(|(init, obj), mut buffer| {
            unsafe {
                buffer.set_buf_init(init);
            }
            ((init, obj), buffer)
        })
    }
}

#[inline]
fn winapi_result() -> Poll<io::Result<usize>> {
    let error = unsafe { GetLastError() };
    assert_ne!(error, 0);
    match error {
        ERROR_IO_PENDING => Poll::Pending,
        ERROR_IO_INCOMPLETE | ERROR_HANDLE_EOF | ERROR_PIPE_CONNECTED | ERROR_NO_DATA => {
            Poll::Ready(Ok(0))
        }
        _ => Poll::Ready(Err(io::Error::from_raw_os_error(error as _))),
    }
}

#[inline]
pub(crate) fn win32_result(res: i32) -> Poll<io::Result<usize>> {
    if res == 0 {
        winapi_result()
    } else {
        Poll::Pending
    }
}

#[inline]
pub(crate) fn winsock_result(res: i32) -> Poll<io::Result<usize>> {
    if res != 0 {
        winapi_result()
    } else {
        Poll::Pending
    }
}
