use std::{
    async_iter::AsyncIterator,
    pin::Pin,
    task::{Context, Poll},
};

use futures_util::Stream;

macro_rules! stream {
    ($($t:tt)*) => {
        $crate::stream::AsyncIteratorStream::new(async gen { $($t)* })
    }
}

pub(crate) use stream;

pub(crate) struct AsyncIteratorStream<I>(I);

impl<I> AsyncIteratorStream<I> {
    pub fn new(iter: I) -> Self {
        Self(iter)
    }
}

impl<I: AsyncIterator> Stream for AsyncIteratorStream<I> {
    type Item = I::Item;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        unsafe { self.map_unchecked_mut(|this| &mut this.0) }.poll_next(cx)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}
