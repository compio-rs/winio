use std::{
    fmt::Debug,
    future::Future,
    pin::Pin,
    sync::{Arc, Mutex},
    task::{Context, Poll},
};

use futures_util::task::AtomicWaker;
use smallvec::SmallVec;

struct ChannelInner<T> {
    data: Mutex<SmallVec<[T; 1]>>,
    waker: AtomicWaker,
}

pub struct Channel<T>(Arc<ChannelInner<T>>);

impl<T> Channel<T> {
    pub fn new() -> Self {
        Self(Arc::new(ChannelInner {
            data: Mutex::default(),
            waker: AtomicWaker::new(),
        }))
    }

    pub fn send(&self, data: T) {
        self.0.data.lock().unwrap().push(data);
        self.0.waker.wake();
    }

    pub fn wait(&self) -> impl Future<Output = ()> + '_ {
        RecvFut(&self.0)
    }

    pub fn fetch_all(&self) -> SmallVec<[T; 1]> {
        std::mem::take(&mut *self.0.data.lock().unwrap())
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

struct RecvFut<'a, T>(&'a ChannelInner<T>);

impl<T> Future for RecvFut<'_, T> {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.0.data.lock().unwrap().is_empty() {
            self.0.waker.register(cx.waker());
            Poll::Pending
        } else {
            Poll::Ready(())
        }
    }
}
