use std::{
    io,
    os::fd::{AsFd, OwnedFd},
    sync::Arc,
    time::Duration,
};

use android_activity::ndk::looper::{FdEvent, ForeignLooper};
use rustix::{
    fs::Timespec,
    time::{
        Itimerspec, TimerfdClockId, TimerfdFlags, TimerfdTimerFlags, timerfd_create,
        timerfd_gettime, timerfd_settime,
    },
};
use winio_callback::SyncCallback;

use crate::Result;

#[derive(Debug)]
pub struct Timer {
    looper: ForeignLooper,
    timer: OwnedFd,
    interval: Itimerspec,
    callback: Arc<SyncCallback>,
}

impl Timer {
    pub fn new(interval: Duration) -> Result<Self> {
        let looper = ForeignLooper::for_thread()
            .ok_or_else(|| io::Error::other("looper not initialized"))?;
        let interval = Itimerspec {
            it_interval: Timespec::default(),
            it_value: Timespec::try_from(interval)
                .map_err(io::Error::other)?
                .max(Timespec {
                    tv_sec: 0,
                    tv_nsec: 1,
                }),
        };
        let timer = timerfd_create(
            TimerfdClockId::Monotonic,
            TimerfdFlags::CLOEXEC | TimerfdFlags::NONBLOCK,
        )
        .map_err(io::Error::from)?;
        Ok(Self {
            looper,
            timer,
            interval,
            callback: Arc::new(SyncCallback::new()),
        })
    }

    pub fn start(&mut self) -> Result<()> {
        let callback = self.callback.clone();
        self.looper
            .add_fd_with_callback(self.timer.as_fd(), FdEvent::INPUT, move |_, _| {
                callback.signal(());
                true
            })
            .map_err(io::Error::other)?;
        timerfd_settime(&self.timer, TimerfdTimerFlags::empty(), &self.interval)
            .map_err(io::Error::from)?;
        Ok(())
    }

    pub fn stop(&mut self) -> Result<()> {
        let new_value = Itimerspec {
            it_interval: Timespec::default(),
            it_value: Timespec::default(),
        };
        timerfd_settime(&self.timer, TimerfdTimerFlags::empty(), &new_value)
            .map_err(io::Error::from)?;
        self.looper
            .remove_fd(self.timer.as_fd())
            .map_err(io::Error::other)?;
        Ok(())
    }

    pub fn is_enabled(&self) -> Result<bool> {
        let interval = timerfd_gettime(self.timer.as_fd()).map_err(io::Error::from)?;
        Ok(interval.it_value.tv_sec > 0 || interval.it_value.tv_nsec > 0)
    }

    pub async fn wait(&self) {
        self.callback.wait().await
    }
}
