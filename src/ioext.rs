use std::{
    future::Future,
    io,
    mem::ManuallyDrop,
    ops::DerefMut,
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
    Networking::WinSock::{LPLOOKUPSERVICE_COMPLETION_ROUTINE, LPWSAOVERLAPPED_COMPLETION_ROUTINE},
    System::IO::{LPOVERLAPPED_COMPLETION_ROUTINE, OVERLAPPED},
};

async fn with_overlapped_impl<C, B>(
    f: impl FnOnce(&mut OVERLAPPED, Option<C>, &mut B) -> Poll<io::Result<usize>>,
    callback: C,
    buffer: B,
) -> BufResult<usize, B> {
    let mut fut = OverlappedFuture::new(buffer);
    let inner = &mut fut.inner;
    let inner = inner.deref_mut();
    match f(
        &mut inner.base,
        Some(callback),
        inner.buffer.as_mut().unwrap(),
    ) {
        Poll::Ready(res) => {
            debug!("operation ready: {:?}", res);
            BufResult(res, inner.buffer.take().unwrap())
        }
        Poll::Pending => {
            debug!("operation pending...");
            fut.await
        }
    }
}

pub(crate) async fn with_overlapped<B>(
    f: impl FnOnce(&mut OVERLAPPED, LPOVERLAPPED_COMPLETION_ROUTINE, &mut B) -> Poll<io::Result<usize>>,
    buffer: B,
) -> BufResult<usize, B> {
    with_overlapped_impl(f, overlapped_callback, buffer).await
}

pub(crate) async fn with_wsa_overlapped<B>(
    f: impl FnOnce(
        &mut OVERLAPPED,
        LPWSAOVERLAPPED_COMPLETION_ROUTINE,
        &mut B,
    ) -> Poll<io::Result<usize>>,
    buffer: B,
) -> BufResult<usize, B> {
    with_overlapped_impl(f, overlapped_wsa_callback, buffer).await
}

pub(crate) async fn with_gai<B>(
    f: impl FnOnce(
        &mut OVERLAPPED,
        LPLOOKUPSERVICE_COMPLETION_ROUTINE,
        &mut B,
    ) -> Poll<io::Result<usize>>,
    buffer: B,
) -> BufResult<usize, B> {
    with_overlapped_impl(f, overlapped_gai_callback, buffer).await
}

enum OverlappedFutureState {
    Active(Option<Waker>),
    Completed(io::Result<usize>),
    Fused,
}

#[repr(C)]
struct OverlappedInner<B> {
    base: OVERLAPPED,
    state: OverlappedFutureState,
    buffer: Option<B>,
}

impl<B> OverlappedInner<B> {
    pub fn new(buffer: B) -> Self {
        Self {
            base: unsafe { std::mem::zeroed() },
            state: OverlappedFutureState::Active(None),
            buffer: Some(buffer),
        }
    }
}

struct OverlappedFuture<B> {
    inner: ManuallyDrop<Box<OverlappedInner<B>>>,
}

impl<B> OverlappedFuture<B> {
    pub fn new(buffer: B) -> Self {
        Self {
            inner: ManuallyDrop::new(Box::new(OverlappedInner::new(buffer))),
        }
    }
}

impl<B> Future for OverlappedFuture<B> {
    type Output = BufResult<usize, B>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let state = std::mem::replace(&mut self.inner.state, OverlappedFutureState::Fused);
        match state {
            OverlappedFutureState::Active(_) => {
                self.inner.state = OverlappedFutureState::Active(Some(cx.waker().clone()));
                Poll::Pending
            }
            OverlappedFutureState::Completed(res) => {
                Poll::Ready(BufResult(res, self.inner.buffer.take().unwrap()))
            }
            OverlappedFutureState::Fused => unreachable!(),
        }
    }
}

impl<B> Drop for OverlappedFuture<B> {
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
    let ptr = lpoverlapped.cast::<OverlappedInner<()>>();
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

unsafe extern "system" fn overlapped_gai_callback(
    dwerror: u32,
    dwbytes: u32,
    lpoverlapped: *const OVERLAPPED,
) {
    overlapped_callback(dwerror, dwbytes, lpoverlapped.cast_mut())
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
