use std::{
    io,
    ops::Deref,
    os::fd::{AsRawFd, RawFd},
    pin::Pin,
    task::{Context, Poll},
    time::Duration,
};

use compio::{compat::Adapter, runtime::Runtime};

pub struct CompioAdapter {
    runtime: Runtime,
}

impl Deref for CompioAdapter {
    type Target = Runtime;

    fn deref(&self) -> &Self::Target {
        &self.runtime
    }
}

impl Adapter for CompioAdapter {
    fn new(runtime: Runtime) -> io::Result<Self> {
        Ok(Self { runtime })
    }

    async fn wait(&self, timeout: Option<Duration>) -> io::Result<()> {
        WaitFuture::new(self.runtime.as_raw_fd(), timeout).await
    }

    fn clear(&self) -> io::Result<()> {
        Ok(())
    }
}

struct WaitFuture {
    fd: RawFd,
    timeout: Option<Duration>,
    polled: bool,
}

impl WaitFuture {
    fn new(fd: RawFd, timeout: Option<Duration>) -> Self {
        Self {
            fd,
            timeout,
            polled: false,
        }
    }
}

impl Future for WaitFuture {
    type Output = io::Result<()>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.polled {
            Poll::Ready(Ok(()))
        } else {
            self.polled = true;
            crate::APP.with(|app| app.register_fd(self.fd, self.timeout, cx.waker().clone()));
            Poll::Pending
        }
    }
}

impl Drop for WaitFuture {
    fn drop(&mut self) {
        crate::APP.with(|app| app.unregister_fd());
    }
}
