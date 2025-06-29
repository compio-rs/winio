//! A callback helper for async.

#![cfg_attr(docsrs, feature(doc_cfg, doc_auto_cfg))]
#![warn(missing_docs)]

use std::{
    cell::RefCell,
    future::poll_fn,
    hint::unreachable_unchecked,
    task::{Poll, Waker},
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
    /// waiting.
    pub fn signal<R: Runnable>(&self, v: T) -> bool {
        let mut state = self.0.borrow_mut();
        match &*state {
            WakerState::Inactive => true,
            WakerState::Signaled(_) => {
                *state = WakerState::Signaled(v);
                true
            }
            WakerState::Active(waker) => {
                waker.wake_by_ref();
                *state = WakerState::Signaled(v);
                drop(state);
                R::run();
                false
            }
        }
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
    pub async fn wait(&self) -> T {
        poll_fn(|cx| self.register(cx.waker())).await
    }
}
