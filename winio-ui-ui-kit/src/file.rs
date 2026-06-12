use std::{
    os::fd::{BorrowedFd, FromRawFd, IntoRawFd},
    path::Path,
};

use compio::{
    BufResult,
    buf::{IoBuf, IoBufMut},
    io::{AsyncReadAt, AsyncWriteAt},
};
use objc2::rc::Retained;
use objc2_foundation::{NSFileHandle, NSString, NSURL};
use winio_ui_apple_common::from_nsstring;

use crate::{Error, Result, catch};

#[derive(Debug)]
pub struct UriFile {
    inner: compio::fs::File,
    uri: Retained<NSURL>,
}

impl AsyncReadAt for UriFile {
    async fn read_at<T: IoBufMut>(&self, buf: T, pos: u64) -> BufResult<usize, T> {
        self.inner.read_at(buf, pos).await
    }
}

impl AsyncWriteAt for UriFile {
    async fn write_at<T: IoBuf>(&mut self, buf: T, pos: u64) -> BufResult<usize, T> {
        self.inner.write_at(buf, pos).await
    }
}

impl AsyncWriteAt for &UriFile {
    async fn write_at<T: IoBuf>(&mut self, buf: T, pos: u64) -> BufResult<usize, T> {
        (&self.inner).write_at(buf, pos).await
    }
}

impl UriFile {
    fn new(file_handle: Retained<NSFileHandle>, uri: Retained<NSURL>) -> Result<Self> {
        let fd = catch(|| file_handle.fileDescriptor())?;
        let fd = unsafe { BorrowedFd::borrow_raw(fd) };
        let fd = fd.try_clone_to_owned()?;
        Ok(Self {
            inner: unsafe { compio::fs::File::from_raw_fd(fd.into_raw_fd()) },
            uri,
        })
    }
}

impl Drop for UriFile {
    fn drop(&mut self) {
        unsafe {
            self.uri.stopAccessingSecurityScopedResource();
        }
    }
}

enum OpenMode {
    Read,
    Write,
    ReadWrite,
}

fn uri_from_path(path: &Path) -> Result<Retained<NSURL>> {
    catch(|| {
        let uri_str = NSString::from_str(&path.to_string_lossy());
        let uri = if let Some(uri) = NSURL::URLWithString(&uri_str)
            && uri
                .scheme()
                .map(|scheme| !scheme.is_empty())
                .unwrap_or_default()
        {
            uri
        } else {
            NSURL::from_path(path, false, None).ok_or_else(|| Error::NullPointer)?
        };
        Ok(uri)
    })
    .flatten()
}

fn open_uri_with_mode(path: &Path, mode: OpenMode) -> Result<UriFile> {
    let uri = uri_from_path(path)?;
    catch(|| {
        if unsafe { !uri.startAccessingSecurityScopedResource() } {
            return Err(Error::Io(std::io::ErrorKind::PermissionDenied.into()));
        }
        let file_handle = match mode {
            OpenMode::Read => NSFileHandle::fileHandleForReadingFromURL_error(&uri)?,
            OpenMode::Write => NSFileHandle::fileHandleForWritingToURL_error(&uri)?,
            OpenMode::ReadWrite => NSFileHandle::fileHandleForUpdatingURL_error(&uri)?,
        };
        UriFile::new(file_handle, uri)
    })
    .flatten()
}

pub async fn open_uri(uri: &Path) -> Result<UriFile> {
    open_uri_with_mode(uri, OpenMode::Read)
}

pub async fn create_uri(uri: &Path) -> Result<UriFile> {
    open_uri_with_mode(uri, OpenMode::Write)
}

pub async fn update_uri(uri: &Path) -> Result<UriFile> {
    open_uri_with_mode(uri, OpenMode::ReadWrite)
}

pub use std::fs::{DirEntry as UriDirEntry, FileType as UriFileType};

#[derive(Debug)]
pub struct UriReadDir {
    inner: std::fs::ReadDir,
    uri: Retained<NSURL>,
}

impl UriReadDir {
    fn new(uri: Retained<NSURL>) -> Result<Self> {
        catch(|| {
            if unsafe { !uri.startAccessingSecurityScopedResource() } {
                return Err(Error::Io(std::io::ErrorKind::PermissionDenied.into()));
            }
            let path = uri.path().ok_or_else(|| Error::NullPointer)?;
            let path = from_nsstring(&path);
            let inner = std::fs::read_dir(path)?;
            Ok(Self { inner, uri })
        })
        .flatten()
    }
}

impl Drop for UriReadDir {
    fn drop(&mut self) {
        unsafe {
            self.uri.stopAccessingSecurityScopedResource();
        }
    }
}

impl Iterator for UriReadDir {
    type Item = Result<UriDirEntry>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|res| res.map_err(Error::from))
    }
}

pub fn read_dir(uri: &Path) -> Result<UriReadDir> {
    let uri = uri_from_path(uri)?;
    UriReadDir::new(uri)
}
