use std::{
    io,
    ops::Deref,
    os::fd::{AsFd, AsRawFd, BorrowedFd, OwnedFd},
    sync::{Arc, Mutex},
    time::Duration,
};

use android_activity::ndk::looper::{FdEvent, ForeignLooper};
use compio::{compat::Adapter, runtime::Runtime};
use rustix::{
    fs::Timespec,
    time::{
        Itimerspec, TimerfdClockId, TimerfdFlags, TimerfdTimerFlags, timerfd_create,
        timerfd_settime,
    },
};

pub struct CompioAdapter {
    runtime: Runtime,
    timer: OwnedFd,
    looper: ForeignLooper,
}

impl CompioAdapter {
    fn as_fd(&self) -> BorrowedFd<'_> {
        unsafe { BorrowedFd::borrow_raw(self.runtime.as_raw_fd()) }
    }
}

impl Deref for CompioAdapter {
    type Target = Runtime;

    fn deref(&self) -> &Self::Target {
        &self.runtime
    }
}

impl Adapter for CompioAdapter {
    fn new(runtime: Runtime) -> io::Result<Self> {
        let looper = ForeignLooper::for_thread()
            .ok_or_else(|| io::Error::other("looper not initialized"))?;
        let timer = timerfd_create(
            TimerfdClockId::Monotonic,
            TimerfdFlags::CLOEXEC | TimerfdFlags::NONBLOCK,
        )?;
        Ok(Self {
            runtime,
            timer,
            looper,
        })
    }

    async fn wait(&self, timeout: Option<Duration>) -> io::Result<()> {
        if timeout == Some(Duration::ZERO) {
            return Ok(());
        }
        let (tx, rx) = oneshot::async_channel();
        let tx = Arc::new(Mutex::new(Some(tx)));
        if let Some(timeout) = timeout {
            let new_value = Itimerspec {
                it_interval: Timespec::default(),
                it_value: Timespec::try_from(timeout)
                    .map_err(io::Error::other)?
                    .max(Timespec {
                        tv_sec: 0,
                        tv_nsec: 1,
                    }),
            };
            timerfd_settime(&self.timer, TimerfdTimerFlags::empty(), &new_value)?;
            let tx = tx.clone();
            self.looper
                .add_fd_with_callback(self.timer.as_fd(), FdEvent::INPUT, move |_, _| {
                    if let Some(tx) = tx.lock().unwrap().take() {
                        tx.send(false).ok();
                    }
                    false
                })
                .map_err(io::Error::other)?;
        }
        self.looper
            .add_fd_with_callback(self.as_fd(), FdEvent::INPUT, move |_, _| {
                if let Some(tx) = tx.lock().unwrap().take() {
                    tx.send(true).ok();
                }
                false
            })
            .map_err(io::Error::other)?;
        if !rx.await.map_err(io::Error::other)? {
            return Err(io::ErrorKind::TimedOut.into());
        }
        Ok(())
    }

    fn clear(&self) -> io::Result<()> {
        let new_value = Itimerspec {
            it_interval: Timespec::default(),
            it_value: Timespec::default(),
        };
        timerfd_settime(&self.timer, TimerfdTimerFlags::empty(), &new_value)?;
        self.looper
            .remove_fd(self.timer.as_fd())
            .map_err(io::Error::other)?;
        self.looper
            .remove_fd(self.as_fd())
            .map_err(io::Error::other)?;
        Ok(())
    }
}
