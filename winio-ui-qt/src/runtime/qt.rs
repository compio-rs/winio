use std::{
    cell::RefCell,
    future::Future,
    os::fd::RawFd,
    task::{RawWaker, RawWakerVTable, Waker},
    time::Duration,
};

use cxx::UniquePtr;

use crate::Result;

#[cxx::bridge]
mod ffi {
    unsafe extern "C++-unwind" {
        include!("winio-ui-qt/src/runtime/qt.hpp");

        type WinioQtEventLoop;

        fn new_event_loop(args: Vec<String>) -> Result<UniquePtr<WinioQtEventLoop>>;

        fn setAppName(self: Pin<&mut Self>, name: &str) -> Result<()>;

        unsafe fn registerFd(self: Pin<&mut Self>, fd: i32, timeout: i32, callback: unsafe fn());
        fn unregisterFd(self: Pin<&mut Self>);

        fn event_loop_wake_up();
        fn event_loop_process();
    }
}

pub struct App {
    event_loop: RefCell<UniquePtr<ffi::WinioQtEventLoop>>,
    waker: RefCell<Option<Waker>>,
}

impl App {
    pub fn new() -> Result<Self> {
        let args = std::env::args().collect::<Vec<_>>();
        let event_loop = RefCell::new(ffi::new_event_loop(args)?);

        Ok(Self {
            event_loop,
            waker: RefCell::new(None),
        })
    }

    pub fn set_app_id(&mut self, id: &str) -> Result<()> {
        self.event_loop.borrow_mut().pin_mut().setAppName(id)?;
        Ok(())
    }

    #[allow(dead_code)]
    pub(crate) fn register_fd(&self, fd: RawFd, timeout: Option<Duration>, waker: Waker) {
        fn callback() {
            if APP.is_set() {
                APP.with(|app| {
                    if let Some(waker) = app.waker.borrow_mut().take() {
                        waker.wake();
                    }
                })
            }
        }

        self.waker.borrow_mut().replace(waker);

        let timeout = timeout.map(|t| t.as_millis() as i32).unwrap_or(-1);
        unsafe {
            self.event_loop
                .borrow_mut()
                .pin_mut()
                .registerFd(fd, timeout, callback);
        }
    }

    #[allow(dead_code)]
    pub(crate) fn unregister_fd(&self) {
        self.event_loop.borrow_mut().pin_mut().unregisterFd();
    }

    pub fn block_on<F: Future>(&self, future: F) -> F::Output {
        APP.set(self, || {
            winio_pollable::block_on(future, qt_waker(), ffi::event_loop_process)
        })
    }
}

scoped_tls::scoped_thread_local!(pub(crate) static APP: App);

fn qt_waker() -> Waker {
    unsafe { Waker::from_raw(qt_raw_waker()) }
}

fn qt_raw_waker() -> RawWaker {
    RawWaker::new(
        std::ptr::null(),
        &RawWakerVTable::new(
            qt_waker_clone,
            qt_waker_wake,
            qt_waker_wake_by_ref,
            qt_waker_drop,
        ),
    )
}

unsafe fn qt_waker_clone(_: *const ()) -> RawWaker {
    qt_raw_waker()
}

unsafe fn qt_waker_wake(_: *const ()) {
    ffi::event_loop_wake_up();
}

unsafe fn qt_waker_wake_by_ref(_: *const ()) {
    ffi::event_loop_wake_up();
}

unsafe fn qt_waker_drop(_: *const ()) {}
