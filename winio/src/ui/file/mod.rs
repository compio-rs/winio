use std::{
    ffi::OsString,
    path::{Path, PathBuf},
};

use compio::{
    BufResult,
    buf::{IoBuf, IoBufMut},
    io::{AsyncReadAt, AsyncWriteAt},
};

use crate::Result;

cfg_if::cfg_if! {
    if #[cfg(any(target_os = "android", target_os = "ios"))] {
        use crate::sys as internal;
    } else {
        #[path = "desktop.rs"]
        mod internal;
    }
}

/// A file opened from a URI. The URI is obtained from
/// [`FileBox`](crate::ui::FileBox).
#[derive(Debug)]
pub struct UriFile {
    inner: internal::UriFile,
}

impl UriFile {
    /// Opens a file from a URI.
    pub async fn open(uri: impl AsRef<Path>) -> Result<Self> {
        Ok(Self {
            inner: internal::open_uri(uri.as_ref()).await?,
        })
    }

    /// Creates a file from a URI. If the file already exists, it will be
    /// truncated.
    pub async fn create(uri: impl AsRef<Path>) -> Result<Self> {
        Ok(Self {
            inner: internal::create_uri(uri.as_ref()).await?,
        })
    }

    /// Opens a file from a URI for both reading and writing. The file must
    /// exist.
    pub async fn update(uri: impl AsRef<Path>) -> Result<Self> {
        Ok(Self {
            inner: internal::update_uri(uri.as_ref()).await?,
        })
    }
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

/// Read a URI directory.
#[derive(Debug)]
pub struct UriReadDir(internal::UriReadDir);

impl Iterator for UriReadDir {
    type Item = Result<UriDirEntry>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|res| Ok(UriDirEntry(res?)))
    }
}

/// A directory entry in a URI directory.
#[derive(Debug)]
pub struct UriDirEntry(internal::UriDirEntry);

impl UriDirEntry {
    /// The URI or the path of the directory entry.
    pub fn path(&self) -> PathBuf {
        self.0.path()
    }

    /// The file name of the directory entry.
    pub fn file_name(&self) -> OsString {
        self.0.file_name()
    }

    /// The file type of the directory entry.
    pub fn file_type(&self) -> Result<UriFileType> {
        Ok(UriFileType(self.0.file_type()?))
    }
}

/// File type of a URI directory entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct UriFileType(internal::UriFileType);

impl UriFileType {
    /// Returns `true` if the file type is a directory.
    pub fn is_dir(&self) -> bool {
        self.0.is_dir()
    }

    /// Returns `true` if the file type is a regular file.
    pub fn is_file(&self) -> bool {
        self.0.is_file()
    }

    /// Returns `true` if the file type is a symbolic link.
    pub fn is_symlink(&self) -> bool {
        self.0.is_symlink()
    }
}

/// Read a URI directory.
pub fn read_uri_dir(uri: impl AsRef<Path>) -> Result<UriReadDir> {
    Ok(UriReadDir(internal::read_dir(uri.as_ref())?))
}
