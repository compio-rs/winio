use std::path::PathBuf;

use gtk4::{
    gio::prelude::FileExt,
    glib::{GString, object::Cast},
};

use crate::{AsRawWindow, AsWindow, Window};

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

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn pattern(&self) -> &str {
        &self.pattern
    }
}

impl From<(&str, &str)> for FileFilter {
    fn from((name, pattern): (&str, &str)) -> Self {
        Self::new(name, pattern)
    }
}

#[derive(Default, Clone)]
pub struct FileBox {
    title: GString,
    filename: GString,
    filters: Vec<FileFilter>,
}

impl FileBox {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn title(mut self, title: impl AsRef<str>) -> Self {
        self.title = GString::from(title.as_ref().to_string());
        self
    }

    pub fn filename(mut self, filename: impl AsRef<str>) -> Self {
        self.filename = GString::from(filename.as_ref().to_string());
        self
    }

    pub fn filters(mut self, filters: impl IntoIterator<Item = FileFilter>) -> Self {
        self.filters = filters.into_iter().collect();
        self
    }

    pub fn add_filter(mut self, filter: impl Into<FileFilter>) -> Self {
        self.filters.push(filter.into());
        self
    }

    pub async fn open(self, parent: Option<impl AsWindow>) -> Option<PathBuf> {
        self.filebox()
            .open_future(parent.map(|w| w.as_window().as_raw_window()).as_ref())
            .await
            .ok()
            .and_then(|f| f.path())
    }

    pub async fn open_multiple(self, parent: Option<&Window>) -> Vec<PathBuf> {
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

    pub async fn save(self, parent: Option<&Window>) -> Option<PathBuf> {
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
