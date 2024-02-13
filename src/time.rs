use std::{
    error::Error,
    fmt::Display,
    future::Future,
    io,
    sync::atomic::{AtomicUsize, Ordering},
    time::{Duration, Instant},
};

use futures_util::FutureExt;
use windows_sys::Win32::UI::WindowsAndMessaging::{KillTimer, SetTimer, WM_TIMER};

use crate::{syscall_bool, ui::AsRawWindow, wait};

pub struct Interval<W: AsRawWindow> {
    wnd: W,
    id: usize,
}

impl<W: AsRawWindow> Interval<W> {
    pub fn new(wnd: W, interval: Duration) -> io::Result<Self> {
        static ID_EVENT: AtomicUsize = AtomicUsize::new(1);

        let id = ID_EVENT.fetch_add(1, Ordering::AcqRel);
        let res = unsafe { SetTimer(wnd.as_raw_window(), id, interval.as_millis() as _, None) };
        if res != 0 {
            Ok(Self { wnd, id })
        } else {
            Err(io::Error::last_os_error())
        }
    }

    pub async fn tick(&mut self) {
        loop {
            let msg = unsafe { wait(self.wnd.as_raw_window(), WM_TIMER) }.await;
            if msg.wParam == self.id {
                break;
            }
        }
    }
}

impl<W: AsRawWindow> Drop for Interval<W> {
    fn drop(&mut self) {
        syscall_bool(unsafe { KillTimer(self.wnd.as_raw_window(), self.id) }).unwrap();
    }
}

pub async fn sleep(duration: Duration, wnd: impl AsRawWindow) {
    let mut timer = Interval::new(wnd, duration).expect("cannot create timer");
    timer.tick().await;
}

pub async fn sleep_until(deadline: Instant, wnd: impl AsRawWindow) {
    sleep(deadline - Instant::now(), wnd).await
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Elapsed;

impl Display for Elapsed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("deadline has elapsed")
    }
}

impl Error for Elapsed {}

pub async fn timeout<F: Future>(
    duration: Duration,
    wnd: impl AsRawWindow,
    future: F,
) -> Result<F::Output, Elapsed> {
    futures_util::select! {
        res = future.fuse() => Ok(res),
        _ = sleep(duration, wnd).fuse() => Err(Elapsed),
    }
}

pub async fn timeout_at<F: Future>(
    deadline: Instant,
    wnd: impl AsRawWindow,
    future: F,
) -> Result<F::Output, Elapsed> {
    timeout(deadline - Instant::now(), wnd, future).await
}

pub fn interval<W: AsRawWindow>(period: Duration, wnd: W) -> Interval<W> {
    Interval::new(wnd, period).expect("cannot create timer")
}
