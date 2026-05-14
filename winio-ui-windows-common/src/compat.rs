use std::{
    io,
    ops::Deref,
    pin::Pin,
    task::{Context, Poll},
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
            crate::set_context(self.handle, self.timeout, cx.waker().clone());
            self.polled = true;
            Poll::Pending
        }
    }
}

impl Drop for HandleFuture {
    fn drop(&mut self) {
        crate::reset_context();
    }
}
