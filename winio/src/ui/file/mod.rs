use std::path::Path;

use compio::{BufResult, buf::IoBufMut, io::AsyncReadAt};

use crate::Result;

cfg_if::cfg_if! {
    if #[cfg(any(target_os = "android", target_os = "ios"))] {
        #[path = "mobile.rs"]
        mod internal;
    } else {
        #[path = "desktop.rs"]
        mod internal;
    }
}

/// A file opened from a URI. The URI is obtained from
/// [`FileBox`](crate::ui::FileBox).
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
