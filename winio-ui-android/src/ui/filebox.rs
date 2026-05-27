use std::path::PathBuf;

use winio_handle::MaybeBorrowedWindow;

use crate::Result;

#[derive(Debug, Default, Clone)]
pub struct FileBox;

impl FileBox {
    pub fn new() -> Self {
        todo!()
    }

    pub fn title(&mut self, _title: impl AsRef<str>) {
        todo!()
    }

    pub fn filename(&mut self, _filename: impl AsRef<str>) {
        todo!()
    }

    pub fn filters(&mut self, _filters: impl IntoIterator<Item = FileFilter>) {
        todo!()
    }

    pub fn add_filter(&mut self, _filter: FileFilter) {
        todo!()
    }

    pub async fn open(
        self,
        _parent: impl Into<MaybeBorrowedWindow<'_>>,
    ) -> Result<Option<PathBuf>> {
        todo!()
    }

    pub async fn open_multiple(
        self,
        _parent: impl Into<MaybeBorrowedWindow<'_>>,
    ) -> Result<Vec<PathBuf>> {
        todo!()
    }

    pub async fn open_folder(
        self,
        _parent: impl Into<MaybeBorrowedWindow<'_>>,
    ) -> Result<Option<PathBuf>> {
        todo!()
    }

    pub async fn save(
        self,
        _parent: impl Into<MaybeBorrowedWindow<'_>>,
    ) -> Result<Option<PathBuf>> {
        todo!()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileFilter;

impl FileFilter {
    pub fn new(_name: impl AsRef<str>, _pattern: impl AsRef<str>) -> Self {
        todo!()
    }
}
