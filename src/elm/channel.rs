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

pub fn channel<T>() -> (Sender<T>, Receiver<T>) {
    let inner = Arc::new(ChannelInner {
        data: Mutex::new(SmallVec::new()),
        waker: AtomicWaker::new(),
    });
    (Sender(inner.clone()), Receiver(inner))
}

pub struct Sender<T>(Arc<ChannelInner<T>>);

impl<T> Sender<T> {
    pub fn send(&self, data: T) {
        self.0.data.lock().unwrap().push(data);
        self.0.waker.wake();
    }
}

impl<T> Debug for Sender<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Sender").finish_non_exhaustive()
    }
}

impl<T> Clone for Sender<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

pub struct Receiver<T>(Arc<ChannelInner<T>>);

impl<T> Receiver<T> {
    pub fn wait(&self) -> impl Future<Output = ()> + '_ {
        RecvFut(&self.0)
    }

    pub fn fetch_all(&self) -> SmallVec<[T; 1]> {
        std::mem::take(&mut *self.0.data.lock().unwrap())
    }
}

impl<T> Debug for Receiver<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Receiver").finish_non_exhaustive()
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
