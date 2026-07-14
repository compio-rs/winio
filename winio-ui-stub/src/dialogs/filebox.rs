use std::path::PathBuf;

use winio_handle::AsWindow;

use crate::{Result, not_impl, not_impl_fut};

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

    pub fn open(
        self,
        _parent: Option<impl AsWindow>,
    ) -> Result<impl Future<Output = Result<Option<PathBuf>>> + 'static> {
        Ok(not_impl_fut())
    }

    pub fn open_multiple(
        self,
        _parent: Option<impl AsWindow>,
    ) -> Result<impl Future<Output = Result<Vec<PathBuf>>> + 'static> {
        Ok(not_impl_fut())
    }

    pub fn open_folder(
        self,
        _parent: Option<impl AsWindow>,
    ) -> Result<impl Future<Output = Result<Option<PathBuf>>> + 'static> {
        Ok(not_impl_fut())
    }

    pub fn save(
        self,
        _parent: Option<impl AsWindow>,
    ) -> Result<impl Future<Output = Result<Option<PathBuf>>> + 'static> {
        Ok(not_impl_fut())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileFilter;

impl FileFilter {
    pub fn new(_name: impl AsRef<str>, _pattern: impl AsRef<str>) -> Self {
        not_impl()
    }
}
