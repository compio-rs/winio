use std::{
    cell::RefCell,
    future::poll_fn,
    hint::unreachable_unchecked,
    rc::Rc,
    task::{Poll, Waker},
};

#[derive(Debug)]
enum WakerState<T> {
    Inactive,
    Active(Waker),
    Signaled(T),
}

#[derive(Debug, Clone)]
pub struct Callback<T = ()>(Rc<RefCell<WakerState<T>>>);

impl<T> Default for Callback<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Callback<T> {
    pub fn new() -> Self {
        Self(Rc::new(RefCell::new(WakerState::Inactive)))
    }

    pub fn signal(&self, v: T) -> bool {
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
                crate::runtime::RUNTIME.with(|runtime| runtime.run());
                false
            }
        }
    }

    pub fn register(&self, waker: &Waker) -> Poll<T> {
        let mut state = self.0.borrow_mut();
        match &*state {
            WakerState::Signaled(_) => {
                let state = std::mem::replace(&mut *state, WakerState::Inactive);
                let v = if let WakerState::Signaled(v) = state {
                    v
                } else {
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

    pub async fn wait(&self) -> T {
        poll_fn(|cx| self.register(cx.waker())).await
    }
}
