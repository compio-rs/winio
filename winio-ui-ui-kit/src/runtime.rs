use std::task::{RawWaker, RawWakerVTable, Waker};

use dispatch2::DispatchQueue;
use futures_util::FutureExt;
use objc2::MainThreadMarker;
use objc2_ui_kit::UIApplication;

use crate::{Error, Result};

pub struct App {
    mtm: MainThreadMarker,
}

impl App {
    pub fn new() -> Result<Self> {
        let mtm = MainThreadMarker::new().ok_or(Error::NotMainThread)?;
        Ok(Self { mtm })
    }

    pub fn block_on<F: Future>(&self, future: F) -> F::Output {
        let future = future.map(|_| {
            std::process::exit(0);
        });
        winio_pollable::enter_block_on(future, dispatcher_waker(), || {
            UIApplication::main(None, None, self.mtm);
        })
    }
}

fn dispatcher_waker() -> Waker {
    unsafe { Waker::from_raw(dispatcher_raw_waker()) }
}

fn dispatcher_raw_waker() -> RawWaker {
    RawWaker::new(
        std::ptr::null(),
        &RawWakerVTable::new(
            dispatcher_clone,
            dispatcher_wake,
            dispatcher_wake_by_ref,
            dispatcher_drop,
        ),
    )
}

unsafe fn dispatcher_clone(_: *const ()) -> RawWaker {
    dispatcher_raw_waker()
}

unsafe fn dispatcher_wake(data: *const ()) {
    unsafe { dispatcher_wake_by_ref(data) }
}

unsafe fn dispatcher_wake_by_ref(_: *const ()) {
    DispatchQueue::main().exec_async(|| {
        winio_pollable::run_current_task();
    })
}

unsafe fn dispatcher_drop(_: *const ()) {}
