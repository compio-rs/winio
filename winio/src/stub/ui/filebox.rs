use std::path::PathBuf;

use winio_handle::MaybeBorrowedWindow;

use crate::stub::{Result, not_impl};

#[derive(Debug, Default, Clone)]
pub struct FileBox;

impl FileBox {
    pub fn new() -> Self {
        not_impl()
    }

    pub fn title(&mut self, _title: impl AsRef<str>) {
        not_impl()
    }

    pub fn filename(&mut self, _filename: impl AsRef<str>) {
        not_impl()
    }

    pub fn filters(&mut self, _filters: impl IntoIterator<Item = FileFilter>) {
        not_impl()
    }

    pub fn add_filter(&mut self, _filter: impl Into<FileFilter>) {
        not_impl()
    }

    pub async fn open(
        self,
        _parent: impl Into<MaybeBorrowedWindow<'_>>,
    ) -> Result<Option<PathBuf>> {
        not_impl()
    }

    pub async fn open_multiple(
        self,
        _parent: impl Into<MaybeBorrowedWindow<'_>>,
    ) -> Result<Vec<PathBuf>> {
        not_impl()
    }

    pub async fn open_folder(
        self,
        _parent: impl Into<MaybeBorrowedWindow<'_>>,
    ) -> Result<Option<PathBuf>> {
        not_impl()
    }

    pub async fn save(
        self,
        _parent: impl Into<MaybeBorrowedWindow<'_>>,
    ) -> Result<Option<PathBuf>> {
        not_impl()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileFilter;

impl FileFilter {
    pub fn new(_name: impl AsRef<str>, _pattern: impl AsRef<str>) -> Self {
        not_impl()
    }
}
