use std::{
    cell::RefCell,
    io,
    ops::Deref,
    pin::Pin,
    task::{Context, Poll, Waker},
    time::Duration,
};

use compio::{compat::Adapter, driver::AsRawFd, runtime::Runtime};
use windows_sys::Win32::Foundation::HANDLE;

pub struct CompioAdapter {
    runtime: Runtime,
}

impl Adapter for CompioAdapter {
    fn new(runtime: Runtime) -> io::Result<Self> {
        Ok(Self { runtime })
    }

    async fn wait(&self, timeout: Option<Duration>) -> io::Result<()> {
        HandleFuture::new(self.runtime.as_raw_fd(), timeout).await
    }

    fn clear(&self) -> io::Result<()> {
        Ok(())
    }
}

impl Deref for CompioAdapter {
    type Target = Runtime;

    fn deref(&self) -> &Self::Target {
        &self.runtime
    }
}

struct HandleFuture {
    handle: HANDLE,
    timeout: Option<Duration>,
    polled: bool,
}

impl HandleFuture {
    fn new(handle: HANDLE, timeout: Option<Duration>) -> Self {
        Self {
            handle,
            timeout,
            polled: false,
        }
    }
}

impl Future for HandleFuture {
    type Output = io::Result<()>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.polled {
            Poll::Ready(Ok(()))
        } else {
            set_context(self.handle, self.timeout, cx.waker().clone());
            self.polled = true;
            Poll::Pending
        }
    }
}

impl Drop for HandleFuture {
    fn drop(&mut self) {
        reset_context();
    }
}

struct HandleContext {
    handle: HANDLE,
    timeout: Option<Duration>,
    waker: Waker,
}

thread_local! {
    static CONTEXT: RefCell<Option<HandleContext>> = const { RefCell::new(None) };
}

fn set_context(handle: HANDLE, timeout: Option<Duration>, waker: Waker) {
    CONTEXT.with_borrow_mut(|ctx| {
        ctx.replace(HandleContext {
            handle,
            timeout,
            waker,
        })
    });
}

fn reset_context() {
    CONTEXT.with_borrow_mut(|ctx| ctx.take());
}

pub(crate) fn get_handle() -> (Option<HANDLE>, Option<Duration>, Option<Waker>) {
    CONTEXT.with_borrow(|ctx| {
        if let Some(ctx) = ctx.as_ref() {
            (Some(ctx.handle), ctx.timeout, Some(ctx.waker.clone()))
        } else {
            (None, None, None)
        }
    })
}
