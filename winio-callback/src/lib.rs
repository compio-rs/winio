//! A callback helper for async.

#![cfg_attr(docsrs, feature(doc_cfg, doc_auto_cfg))]
#![warn(missing_docs)]

use std::{
    cell::RefCell,
    future::Future,
    hint::unreachable_unchecked,
    pin::Pin,
    task::{Context, Poll, Waker},
};

/// An abstract for global runtime.
pub trait Runnable {
    /// It will be called if the callback is signaled, and there's a waker to be
    /// waked.
    fn run();
}

impl Runnable for () {
    fn run() {}
}

#[derive(Debug)]
enum WakerState<T> {
    Inactive,
    Active(Waker),
    Signaled(T),
}

/// A callback type. It is usually signaled in a GUI widget callback.
#[derive(Debug)]
pub struct Callback<T = ()>(RefCell<WakerState<T>>);

impl<T> Default for Callback<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Callback<T> {
    /// Create [`Callback`].
    pub fn new() -> Self {
        Self(RefCell::new(WakerState::Inactive))
    }

    /// Signal the callback and try to run the runtime if there's a waker
    /// waiting. Returns `true` if not handled.
    pub fn signal<R: Runnable>(&self, v: T) -> bool {
        let mut state = self.0.borrow_mut();
        match &*state {
            WakerState::Inactive => return true,
            WakerState::Signaled(_) => {
                // If a state is signaled again, the runtime might be too busy
                // to wake the waker. Just try to run it again.
            }
            WakerState::Active(waker) => {
                waker.wake_by_ref();
            }
        }
        *state = WakerState::Signaled(v);
        drop(state);
        R::run();
        false
    }

    pub(crate) fn register(&self, waker: &Waker) -> Poll<T> {
        let mut state = self.0.borrow_mut();
        match &*state {
            WakerState::Signaled(_) => {
                let state = std::mem::replace(&mut *state, WakerState::Inactive);
                let v = if let WakerState::Signaled(v) = state {
                    v
                } else {
                    // SAFETY: already checked
                    unsafe { unreachable_unchecked() }
                };
                Poll::Ready(v)
            }
            _ => {
                *state = WakerState::Active(waker.clone());
                Poll::Pending
            }
        }
    }

    /// Wait for signal.
    pub fn wait(&self) -> impl Future<Output = T> + '_ {
        WaitFut(self)
    }
}

struct WaitFut<'a, T>(&'a Callback<T>);

impl<T> Future for WaitFut<'_, T> {
    type Output = T;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.0.register(cx.waker())
    }
}

impl<T> Drop for WaitFut<'_, T> {
    fn drop(&mut self) {
        // Deregister the waker.
        *self.0.0.borrow_mut() = WakerState::Inactive;
    }
}
