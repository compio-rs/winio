use std::path::PathBuf;

use winio_handle::MaybeBorrowedWindow;

use crate::sys;

/// File open/save box.
#[derive(Debug, Default, Clone)]
pub struct FileBox(sys::FileBox);

impl FileBox {
    /// Create [`FileBox`].
    pub fn new() -> Self {
        Self(sys::FileBox::new())
    }

    /// Box title.
    pub fn title(mut self, title: impl AsRef<str>) -> Self {
        self.0.title(title.as_ref());
        self
    }

    /// Default filename.
    pub fn filename(mut self, filename: impl AsRef<str>) -> Self {
        self.0.filename(filename.as_ref());
        self
    }

    /// Set file filters.
    pub fn filters(mut self, filters: impl IntoIterator<Item = FileFilter>) -> Self {
        self.0.filters(filters.into_iter().map(|f| f.0));
        self
    }

    /// Add a file filter.
    pub fn add_filter(mut self, filter: impl Into<FileFilter>) -> Self {
        self.0.add_filter(filter.into().0);
        self
    }

    /// Show open file dialog.
    pub async fn open(self, parent: impl Into<MaybeBorrowedWindow<'_>>) -> Option<PathBuf> {
        self.0.open(parent.into().0).await
    }

    /// Show open file dialog, allowing multiple selection.
    pub async fn open_multiple(self, parent: impl Into<MaybeBorrowedWindow<'_>>) -> Vec<PathBuf> {
        self.0.open_multiple(parent.into().0).await
    }

    /// Show open file dialog, select folder only.
    pub async fn open_folder(self, parent: impl Into<MaybeBorrowedWindow<'_>>) -> Option<PathBuf> {
        self.0.open_folder(parent.into().0).await
    }

    /// Show save file dialog.
    pub async fn save(self, parent: impl Into<MaybeBorrowedWindow<'_>>) -> Option<PathBuf> {
        self.0.save(parent.into().0).await
    }
}

/// File type filter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileFilter(sys::FileFilter);

impl FileFilter {
    /// Create [`FileFilter`].
    ///
    /// The pattern should be like `*.txt` or `*.*`.
    pub fn new(name: impl AsRef<str>, pattern: impl AsRef<str>) -> Self {
        Self(sys::FileFilter::new(name.as_ref(), pattern.as_ref()))
    }
}

impl<S: AsRef<str>, P: AsRef<str>> From<(S, P)> for FileFilter {
    fn from((name, pattern): (S, P)) -> Self {
        Self::new(name, pattern)
    }
}
