use std::path::PathBuf;

use objc2::rc::Retained;
use objc2_foundation::NSString;
use winio_handle::AsWindow;

use crate::{Error, Result};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileFilter {
    name: String,
    pattern: String,
}

impl FileFilter {
    pub fn new(name: &str, pattern: &str) -> Self {
        Self {
            name: name.to_string(),
            pattern: pattern.to_string(),
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct FileBox {
    title: Retained<NSString>,
    filename: Retained<NSString>,
    filters: Vec<FileFilter>,
}

// SAFETY: NSString is thread-safe.
unsafe impl Send for FileBox {}
unsafe impl Sync for FileBox {}

impl FileBox {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn title(&mut self, title: &str) {
        self.title = NSString::from_str(title);
    }

    pub fn filename(&mut self, filename: &str) {
        self.filename = NSString::from_str(filename);
    }

    pub fn filters(&mut self, filters: impl IntoIterator<Item = FileFilter>) {
        self.filters = filters.into_iter().collect();
    }

    pub fn add_filter(&mut self, filter: FileFilter) {
        self.filters.push(filter);
    }

    pub async fn open(self, parent: Option<impl AsWindow>) -> Result<Option<PathBuf>> {
        filebox(parent, &self.filters, false)
            .await?
            .into_iter()
            .next()
            .transpose()
    }

    pub async fn open_multiple(self, parent: Option<impl AsWindow>) -> Result<Vec<PathBuf>> {
        filebox(parent, &self.filters, true)
            .await?
            .into_iter()
            .collect()
    }

    pub async fn open_folder(self, parent: Option<impl AsWindow>) -> Result<Option<PathBuf>> {
        filebox_folder(parent).await
    }

    pub async fn save(self, parent: Option<impl AsWindow>) -> Result<Option<PathBuf>> {
        Ok(None)
    }
}

async fn filebox(
    _parent: Option<impl AsWindow>,
    _filters: &[FileFilter],
    _multiple: bool,
) -> Result<Vec<Result<PathBuf>>> {
    Err(Error::NotSupported)
}

async fn filebox_folder(_parent: Option<impl AsWindow>) -> Result<Option<PathBuf>> {
    Err(Error::NotSupported)
}
