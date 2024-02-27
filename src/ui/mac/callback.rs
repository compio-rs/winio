use std::{
    cell::RefCell,
    future::poll_fn,
    rc::Rc,
    task::{Poll, Waker},
};

#[derive(Debug)]
enum WakerState {
    Inactive,
    Active(Waker),
    Signaled,
}

#[derive(Debug, Clone)]
pub struct Callback(Rc<RefCell<WakerState>>);

impl Callback {
    pub fn new() -> Self {
        Self(Rc::new(RefCell::new(WakerState::Inactive)))
    }

    pub fn signal(&self) -> bool {
        let mut state = self.0.borrow_mut();
        match &*state {
            WakerState::Inactive | WakerState::Signaled => true,
            WakerState::Active(waker) => {
                waker.wake_by_ref();
                *state = WakerState::Signaled;
                false
            }
        }
    }

    pub fn register(&self, waker: &Waker) -> Poll<()> {
        let mut state = self.0.borrow_mut();
        match &*state {
            WakerState::Signaled => Poll::Ready(()),
            _ => {
                *state = WakerState::Active(waker.clone());
                Poll::Pending
            }
        }
    }

    pub async fn wait(&self) {
        poll_fn(|cx| self.register(cx.waker())).await
    }
}
