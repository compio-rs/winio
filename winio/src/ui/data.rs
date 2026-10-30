use winio_primitive::DataEntry;
pub use winio_primitive::DataFormat;

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
    pub fn new(entry: DataEntry<Image>) -> Result<Self> {
        let mut data = Self::from_sys(sys::Data::new()?);
        data.insert(entry)?;
        Ok(data)
    }

    /// Add an entry.
    pub fn insert(&mut self, entry: DataEntry<Image>) -> Result<()> {
        match entry {
            DataEntry::Text(text) => self.inner.insert_text(text),
            DataEntry::Html(html) => self.inner.insert_html(html),
            DataEntry::Image(image) => self.inner.insert_image(&image.0),
            DataEntry::Paths(paths) => self.inner.insert_paths(&paths),
            DataEntry::Unknown(bytes) => self.inner.insert_unknown(bytes),
        }
    }

    /// The available formats, in backend order.
    ///
    /// The iterator corresponds one-to-one with [`IntoIterator`] for this
    /// [`Data`], including multiple [`DataFormat::Unknown`] entries.
    pub fn formats(&self) -> Result<impl Iterator<Item = DataFormat> + '_> {
        self.inner.formats()
    }

    /// Iterate over the entries of the given format.
    ///
    /// Entries of other formats are skipped. A backend may provide multiple
    /// entries for one format.
    pub fn into_format(self, format: DataFormat) -> DataIntoFormat {
        DataIntoFormat {
            inner: self.into_iter(),
            format,
        }
    }
}

type SysEntryResult = Result<winio_primitive::DataEntry<sys::Image>>;

fn map_entry(entry: SysEntryResult) -> Result<DataEntry<Image>> {
    entry.map(|entry| entry.map_image(Image))
}

impl IntoIterator for Data {
    type IntoIter = DataIntoIter;
    type Item = Result<DataEntry<Image>>;

    fn into_iter(self) -> Self::IntoIter {
        let inner = self
            .inner
            .into_iter()
            .map(map_entry as fn(SysEntryResult) -> Result<DataEntry<Image>>);
        DataIntoIter { inner }
    }
}

/// Iterator over the entries of [`Data`].
#[allow(clippy::type_complexity)]
pub struct DataIntoIter {
    inner: std::iter::Map<sys::DataIntoIter, fn(SysEntryResult) -> Result<DataEntry<Image>>>,
}

impl Iterator for DataIntoIter {
    type Item = Result<DataEntry<Image>>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}

/// Iterator over the entries of one [`DataFormat`].
pub struct DataIntoFormat {
    inner: DataIntoIter,
    format: DataFormat,
}

impl Iterator for DataIntoFormat {
    type Item = Result<DataEntry<Image>>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.inner.next()? {
                Ok(entry) if entry.format() == self.format => return Some(Ok(entry)),
                Ok(_) => {}
                Err(error) => return Some(Err(error)),
            }
        }
    }
}
