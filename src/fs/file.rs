use std::os::windows::io::{AsRawHandle, FromRawHandle, IntoRawHandle, RawHandle};

use compio_buf::{BufResult, IoBuf, IoBufMut};
use compio_io::{AsyncReadAt, AsyncWriteAt};

#[derive(Debug)]
pub struct File {
    inner: std::fs::File,
}

impl AsyncReadAt for File {
    async fn read_at<T: IoBufMut>(&self, buf: T, pos: u64) -> BufResult<usize, T> {
        todo!()
    }
}

impl AsyncWriteAt for File {
    async fn write_at<T: IoBuf>(&mut self, buf: T, pos: u64) -> BufResult<usize, T> {
        todo!()
    }
}

impl AsRawHandle for File {
    fn as_raw_handle(&self) -> RawHandle {
        self.inner.as_raw_handle()
    }
}

impl IntoRawHandle for File {
    fn into_raw_handle(self) -> RawHandle {
        self.inner.into_raw_handle()
    }
}

impl FromRawHandle for File {
    unsafe fn from_raw_handle(handle: RawHandle) -> Self {
        Self {
            inner: std::fs::File::from_raw_handle(handle),
        }
    }
}
