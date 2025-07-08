use std::{panic::resume_unwind, path::PathBuf};

use widestring::U16CString;
use windows::Win32::Foundation::HWND;
use winio_handle::AsWindow;
pub use winio_ui_windows_common::FileFilter;
use winio_ui_windows_common::filebox;

#[derive(Debug, Default, Clone)]
pub struct FileBox {
    title: U16CString,
    filename: U16CString,
    filters: Vec<FileFilter>,
}

impl FileBox {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn title(&mut self, title: &str) {
        self.title = U16CString::from_str_truncate(title);
    }

    pub fn filename(&mut self, filename: &str) {
        self.filename = U16CString::from_str_truncate(filename);
    }

    pub fn filters(&mut self, filters: impl IntoIterator<Item = FileFilter>) {
        self.filters = filters.into_iter().collect();
    }

    pub fn add_filter(&mut self, filter: FileFilter) {
        self.filters.push(filter);
    }

    pub async fn open(self, parent: Option<impl AsWindow>) -> Option<PathBuf> {
        let parent = parent.map(|p| p.as_window().as_winui().clone());
        compio::runtime::spawn_blocking(move || unsafe {
            let parent = parent.and_then(|w| Some(HWND(w.AppWindow().ok()?.Id().ok()?.Value as _)));
            filebox(
                parent,
                self.title,
                self.filename,
                self.filters,
                true,
                false,
                false,
            )
            .result()
        })
        .await
        .unwrap_or_else(|e| resume_unwind(e))
    }

    pub async fn open_multiple(self, parent: Option<impl AsWindow>) -> Vec<PathBuf> {
        let parent = parent.map(|p| p.as_window().as_winui().clone());
        compio::runtime::spawn_blocking(move || unsafe {
            let parent = parent.and_then(|w| Some(HWND(w.AppWindow().ok()?.Id().ok()?.Value as _)));
            filebox(
                parent,
                self.title,
                self.filename,
                self.filters,
                true,
                true,
                false,
            )
            .results()
        })
        .await
        .unwrap_or_else(|e| resume_unwind(e))
    }

    pub async fn open_folder(self, parent: Option<impl AsWindow>) -> Option<PathBuf> {
        let parent = parent.map(|p| p.as_window().as_winui().clone());
        compio::runtime::spawn_blocking(move || unsafe {
            let parent = parent.and_then(|w| Some(HWND(w.AppWindow().ok()?.Id().ok()?.Value as _)));
            filebox(
                parent,
                self.title,
                self.filename,
                self.filters,
                true,
                false,
                true,
            )
            .result()
        })
        .await
        .unwrap_or_else(|e| resume_unwind(e))
    }

    pub async fn save(self, parent: Option<impl AsWindow>) -> Option<PathBuf> {
        let parent = parent.map(|p| p.as_window().as_winui().clone());
        compio::runtime::spawn_blocking(move || unsafe {
            let parent = parent.and_then(|w| Some(HWND(w.AppWindow().ok()?.Id().ok()?.Value as _)));
            filebox(
                parent,
                self.title,
                self.filename,
                self.filters,
                false,
                false,
                false,
            )
            .result()
        })
        .await
        .unwrap_or_else(|e| resume_unwind(e))
    }
}
