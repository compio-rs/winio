use std::{fmt::Debug, time::Duration};

use cxx::UniquePtr;
use winio_callback::Callback;
use winio_pollable::GlobalRuntime;

use crate::Result;

pub struct Timer {
    inner: UniquePtr<ffi::QTimer>,
    on_timeout: Box<Callback>,
}

impl Timer {
    pub fn new(interval: Duration) -> Result<Self> {
        let mut inner = ffi::new_timer()?;
        inner.pin_mut().setInterval(interval.as_millis() as _)?;
        let on_timeout = Box::new(Callback::new());
        unsafe {
            ffi::timer_connect_timeout(
                inner.pin_mut(),
                Self::on_timeout,
                on_timeout.as_ref() as *const _ as _,
            )?;
        }
        Ok(Self { inner, on_timeout })
    }

    pub fn start(&mut self) -> Result<()> {
        self.inner.pin_mut().start()?;
        Ok(())
    }

    pub fn stop(&mut self) -> Result<()> {
        self.inner.pin_mut().stop()?;
        Ok(())
    }

    pub fn is_enabled(&self) -> Result<bool> {
        Ok(self.inner.isActive()?)
    }

    fn on_timeout(c: *const u8) {
        let c = c as *const Callback<()>;
        if let Some(c) = unsafe { c.as_ref() } {
            c.signal::<GlobalRuntime>(());
        }
    }

    pub async fn wait(&self) {
        self.on_timeout.wait().await
    }
}

impl Debug for Timer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Timer").finish_non_exhaustive()
    }
}

#[cxx::bridge]
mod ffi {
    unsafe extern "C++-unwind" {
        include!("winio-ui-qt/src/platform/timer.hpp");

        type QTimer;

        fn new_timer() -> Result<UniquePtr<QTimer>>;
        unsafe fn timer_connect_timeout(
            timer: Pin<&mut QTimer>,
            callback: unsafe fn(*const u8),
            data: *const u8,
        ) -> Result<()>;

        fn setInterval(self: Pin<&mut QTimer>, msec: i32) -> Result<()>;
        fn isActive(self: &QTimer) -> Result<bool>;
        fn start(self: Pin<&mut QTimer>) -> Result<()>;
        fn stop(self: Pin<&mut QTimer>) -> Result<()>;
    }
}
