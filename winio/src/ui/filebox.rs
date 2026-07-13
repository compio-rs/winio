use std::path::PathBuf;

use winio_handle::MaybeBorrowedWindow;

use crate::{sys, sys::Result};

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
    ///
    /// This method is not a usual `async fn`. It shows the dialog immediately,
    /// and returns a future that waits for the result. This design allows you
    /// to spawn the returned future.
    ///
    /// ## Platform specific
    /// * AppKit: This method is blocking without parent window.
    pub fn open<'a>(
        self,
        parent: impl Into<MaybeBorrowedWindow<'a>>,
    ) -> Result<impl Future<Output = Result<Option<PathBuf>>> + 'static> {
        self.0.open(parent.into().0)
    }

    /// Show open file dialog, allowing multiple selection.
    ///
    /// This method is not a usual `async fn`. It shows the dialog immediately,
    /// and returns a future that waits for the result. This design allows you
    /// to spawn the returned future.
    ///
    /// ## Platform specific
    /// * AppKit: This method is blocking without parent window.
    pub fn open_multiple<'a>(
        self,
        parent: impl Into<MaybeBorrowedWindow<'a>>,
    ) -> Result<impl Future<Output = Result<Vec<PathBuf>>> + 'static> {
        self.0.open_multiple(parent.into().0)
    }

    /// Show open file dialog, select folder only.
    ///
    /// This method is not a usual `async fn`. It shows the dialog immediately,
    /// and returns a future that waits for the result. This design allows you
    /// to spawn the returned future.
    ///
    /// ## Platform specific
    /// * AppKit: This method is blocking without parent window.
    pub fn open_folder<'a>(
        self,
        parent: impl Into<MaybeBorrowedWindow<'a>>,
    ) -> Result<impl Future<Output = Result<Option<PathBuf>>> + 'static> {
        self.0.open_folder(parent.into().0)
    }

    /// Show save file dialog.
    ///
    /// This method is not a usual `async fn`. It shows the dialog immediately,
    /// and returns a future that waits for the result. This design allows you
    /// to spawn the returned future.
    ///
    /// ## Platform specific
    /// * AppKit: This method is blocking without parent window.
    pub fn save<'a>(
        self,
        parent: impl Into<MaybeBorrowedWindow<'a>>,
    ) -> Result<impl Future<Output = Result<Option<PathBuf>>> + 'static> {
        self.0.save(parent.into().0)
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
