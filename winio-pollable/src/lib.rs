//! A pollable runtime based on [`compio`]. It is just a compio runtime except
//! Linux, where there is also an eventfd if the driver is io-uring. This crate
//! ensures that the raw fd returned by [`RawFd::as_raw_fd`] is always actively
//! pollable.

#![warn(missing_docs)]

use std::{
    cell::RefCell,
    future::Future,
    pin::Pin,
    task::{Context, Poll, Waker},
};

use futures_util::FutureExt;

/// Run the current task that is blocking on. Returns true if the task has
/// completed, and false otherwise.
pub fn run_current_task() -> bool {
    if MAIN_TASK.is_set() {
        MAIN_TASK.with(|task| task.poll())
    } else {
        false
    }
}

/// Set the current task that is going to be blocked on, and run the provided
/// function.
pub fn enter_block_on<F: Future<Output = ()>, T>(
    future: F,
    waker: Waker,
    f: impl FnOnce() -> T,
) -> T {
    let task = unsafe { MainTask::new(future, waker) };
    MAIN_TASK.set(&task, f)
}

/// Block on a future until it completes, while also running the provided poll
/// function to drive the runtime.
pub fn block_on<F: Future>(future: F, waker: Waker, poll: impl Fn()) -> F::Output {
    let result = RefCell::new(None);

    enter_block_on(
        future.map(|res| {
            result.replace(Some(res));
        }),
        waker,
        || {
            loop {
                if !run_current_task() {
                    poll();
                }

                if let Some(result) = result.take() {
                    break result;
                }
            }
        },
    )
}

struct MainTask {
    future: RefCell<Pin<Box<dyn Future<Output = ()>>>>,
    waker: Waker,
}

unsafe fn reduce_lifetime<'a>(
    future: impl Future<Output = ()> + 'a,
) -> Pin<Box<dyn Future<Output = ()> + 'static>> {
    let future = Box::pin(future) as Pin<Box<dyn Future<Output = ()> + 'a>>;
    unsafe {
        std::mem::transmute::<
            Pin<Box<dyn Future<Output = ()> + 'a>>,
            Pin<Box<dyn Future<Output = ()> + 'static>>,
        >(future)
    }
}

impl MainTask {
    pub unsafe fn new(future: impl Future<Output = ()>, waker: Waker) -> Self {
        Self {
            // SAFETY: the future will only be polled within the scope of `enter_block_on`, which
            // guarantees that the future will not outlive the main task.
            future: RefCell::new(unsafe { reduce_lifetime(future.fuse()) }),
            waker,
        }
    }

    pub fn poll(&self) -> bool {
        let mut cx = Context::from_waker(&self.waker);
        if let Ok(mut fut) = self.future.try_borrow_mut() {
            if let Poll::Ready(()) = fut.as_mut().poll(&mut cx) {
                // The future has completed, so we should not wait for the driver to wake us up
                // anymore.
                true
            } else {
                false
            }
        } else {
            false
        }
    }
}

scoped_tls::scoped_thread_local!(static MAIN_TASK: MainTask);

/// An adapter for [`winio_callback::Runnable`] that runs the current task.
#[cfg(feature = "callback")]
pub struct GlobalRuntime;

#[cfg(feature = "callback")]
impl winio_callback::Runnable for GlobalRuntime {
    #[inline]
    fn run() {
        run_current_task();
    }
}
