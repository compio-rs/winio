use futures_util::{Stream, StreamExt};
use winio_primitive::{DataEntry, DataFormat};

use crate::{Result, sys, ui::Image};

/// Data transferred through the clipboard or drag and drop.
#[derive(Debug)]
pub struct Data {
    inner: sys::Data,
}

impl Data {
    pub(crate) fn from_sys(inner: sys::Data) -> Self {
        Self { inner }
    }

    pub(crate) fn sys(&self) -> &sys::Data {
        &self.inner
    }

    /// Create data with a single entry.
    pub async fn new(entry: DataEntry<Image>) -> Result<Self> {
        let mut data = Self::from_sys(sys::Data::new()?);
        data.insert(entry).await?;
        Ok(data)
    }

    /// Add an entry.
    pub async fn insert(&mut self, entry: DataEntry<Image>) -> Result<()> {
        self.inner.insert(entry.map_image(|image| image.0)).await
    }

    /// The available formats, in backend order.
    ///
    /// The iterator corresponds one-to-one with [`Data::into_stream`].
    pub fn formats(&self) -> Result<impl Iterator<Item = DataFormat> + '_> {
        self.inner.formats()
    }

    /// Consume the data as a stream of entries.
    pub fn into_stream(self) -> impl Stream<Item = Result<DataEntry<Image>>> + Unpin {
        self.inner
            .into_stream()
            .map(|entry| entry.map(|entry| entry.map_image(Image)))
    }
}
