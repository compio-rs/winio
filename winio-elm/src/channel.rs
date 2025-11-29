use std::{
    fmt::Debug,
    future::Future,
    pin::Pin,
    sync::{Arc, Mutex},
    task::{Context, Poll, Waker},
};

use smallvec::SmallVec;

struct ChannelInner<T> {
    data: SmallVec<[T; 1]>,
    waker: Option<Waker>,
}

pub struct Channel<T>(Arc<Mutex<ChannelInner<T>>>);

impl<T> Channel<T> {
    pub fn new() -> Self {
        Self(Arc::new(Mutex::new(ChannelInner {
            data: SmallVec::new(),
            waker: None,
        })))
    }

    pub fn send(&self, data: T) {
        let mut inner = self.0.lock().unwrap();
        inner.data.push(data);
        if let Some(waker) = inner.waker.take() {
            waker.wake();
        }
    }

    pub fn wait(&self) -> impl Future<Output = ()> + '_ {
        RecvFut(&self.0)
    }

    pub fn fetch_all(&self) -> SmallVec<[T; 1]> {
        let mut inner = self.0.lock().unwrap();
        std::mem::take(&mut inner.data)
    }
}

impl<T> Debug for Channel<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Channel").finish_non_exhaustive()
    }
}

impl<T> Clone for Channel<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

struct RecvFut<'a, T>(&'a Mutex<ChannelInner<T>>);

impl<T> Future for RecvFut<'_, T> {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut inner = self.0.lock().unwrap();
        if inner.data.is_empty() {
            inner.waker = Some(cx.waker().clone());
            Poll::Pending
        } else {
            Poll::Ready(())
        }
    }
}
