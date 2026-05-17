use std::{io, ops::Deref, os::fd::AsRawFd, time::Duration};

use compio::{compat::Adapter, runtime::Runtime};
use futures_util::TryFutureExt;
use glib_unix::unix_fd_add_local;
use gtk4::glib::{ControlFlow, IOCondition, future_with_timeout};
use local_sync::oneshot;

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
        let (tx, rx) = oneshot::channel();
        let mut send = Some(tx);
        unix_fd_add_local(
            self.runtime.as_raw_fd(),
            IOCondition::IN,
            move |_fd, _cond| {
                let _ = send.take().unwrap().send(());
                ControlFlow::Break
            },
        );
        let fut = rx.map_err(|_| io::ErrorKind::Interrupted.into());
        if let Some(timeout) = timeout {
            future_with_timeout(timeout, fut)
                .await
                .map_err(|_| io::ErrorKind::TimedOut)?
        } else {
            fut.await
        }
    }

    fn clear(&self) -> io::Result<()> {
        Ok(())
    }
}
