use std::path::PathBuf;

use gtk4::{
    gio::prelude::FileExt,
    glib::{GString, object::Cast},
};
use winio_handle::{AsRawWindow, AsWindow};

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
    title: GString,
    filename: GString,
    filters: Vec<FileFilter>,
}

impl FileBox {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn title(&mut self, title: &str) {
        self.title = GString::from(title.to_string());
    }

    pub fn filename(&mut self, filename: &str) {
        self.filename = GString::from(filename.to_string());
    }

    pub fn filters(&mut self, filters: impl IntoIterator<Item = FileFilter>) {
        self.filters = filters.into_iter().collect();
    }

    pub fn add_filter(&mut self, filter: FileFilter) {
        self.filters.push(filter);
    }

    pub async fn open(self, parent: Option<impl AsWindow>) -> Option<PathBuf> {
        self.filebox()
            .open_future(parent.map(|w| w.as_window().as_raw_window()).as_ref())
            .await
            .ok()
            .and_then(|f| f.path())
    }

    pub async fn open_multiple(self, parent: Option<impl AsWindow>) -> Vec<PathBuf> {
        self.filebox()
            .open_multiple_future(parent.map(|w| w.as_window().as_raw_window()).as_ref())
            .await
            .ok()
            .map(|list| {
                list.into_iter()
                    .filter_map(|f| f.ok())
                    .filter_map(|f| f.dynamic_cast::<gtk4::gio::File>().ok())
                    .filter_map(|f| f.path())
                    .collect()
            })
            .unwrap_or_default()
    }

    pub async fn open_folder(self, parent: Option<impl AsWindow>) -> Option<PathBuf> {
        self.filebox()
            .select_folder_future(parent.map(|w| w.as_window().as_raw_window()).as_ref())
            .await
            .ok()
            .and_then(|f| f.path())
    }

    pub async fn save(self, parent: Option<impl AsWindow>) -> Option<PathBuf> {
        self.filebox()
            .save_future(parent.map(|w| w.as_window().as_raw_window()).as_ref())
            .await
            .ok()
            .and_then(|f| f.path())
    }

    fn filebox(self) -> gtk4::FileDialog {
        let filter = gtk4::FileFilter::new();
        if !self.filters.is_empty() {
            for f in self.filters {
                filter.add_pattern(&f.pattern);
            }
        } else {
            filter.add_pattern("*.*");
        }

        gtk4::FileDialog::builder()
            .modal(true)
            .title(self.title)
            .initial_name(self.filename)
            .default_filter(&filter)
            .build()
    }
}
