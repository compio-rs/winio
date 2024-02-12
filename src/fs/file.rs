use std::{
    io,
    os::windows::io::{AsRawHandle, FromRawHandle, IntoRawHandle, RawHandle},
    path::Path,
};

use compio_buf::{BufResult, IoBuf, IoBufMut};
use compio_io::{AsyncReadAt, AsyncWriteAt};
use compio_log::*;
use windows_sys::Win32::Storage::FileSystem::{ReadFileEx, WriteFileEx};

use crate::{fs::OpenOptions, win32_result, with_overlapped, BufResultExt};

#[derive(Debug)]
pub struct File {
    inner: std::fs::File,
}

impl File {
    pub fn open(path: impl AsRef<Path>) -> io::Result<Self> {
        OpenOptions::new().read(true).open(path)
    }

    pub fn create(path: impl AsRef<Path>) -> io::Result<Self> {
        OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(path)
    }
}

impl AsyncReadAt for File {
    async fn read_at<T: IoBufMut>(&self, mut buf: T, pos: u64) -> BufResult<usize, T> {
        debug!("read_at {}", pos);
        let res = with_overlapped(|optr, callback| {
            optr.Anonymous.Anonymous.Offset = (pos & 0xFFFFFFFF) as _;
            optr.Anonymous.Anonymous.OffsetHigh = (pos >> 32) as _;

            let slice = buf.as_mut_slice();
            let res = unsafe {
                ReadFileEx(
                    self.as_raw_handle() as _,
                    slice.as_mut_ptr() as _,
                    slice.len() as _,
                    optr,
                    callback,
                )
            };
            win32_result(res)
        })
        .await;
        BufResult(res, buf).map_advanced()
    }
}

impl AsyncWriteAt for File {
    async fn write_at<T: IoBuf>(&mut self, buf: T, pos: u64) -> BufResult<usize, T> {
        debug!("write_at {}", pos);
        let res = with_overlapped(|optr, callback| {
            optr.Anonymous.Anonymous.Offset = (pos & 0xFFFFFFFF) as _;
            optr.Anonymous.Anonymous.OffsetHigh = (pos >> 32) as _;

            let slice = buf.as_slice();
            let res = unsafe {
                WriteFileEx(
                    self.as_raw_handle() as _,
                    slice.as_ptr() as _,
                    slice.len() as _,
                    optr,
                    callback,
                )
            };
            win32_result(res)
        })
        .await;
        BufResult(res, buf)
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
