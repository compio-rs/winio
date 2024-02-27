use std::{cell::RefCell, io, path::PathBuf};

use icrate::{
    block2::ConcreteBlock,
    objc2::rc::Id,
    AppKit::{NSModalResponseOK, NSOpenPanel, NSSavePanel},
    Foundation::{MainThreadMarker, NSArray, NSString},
};

use super::from_nsstring;
use crate::Window;

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
    title: String,
    filename: String,
    filters: Vec<FileFilter>,
}

impl FileBox {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn title(mut self, title: impl AsRef<str>) -> Self {
        self.title = title.as_ref().to_string();
        self
    }

    pub fn filename(mut self, filename: impl AsRef<str>) -> Self {
        self.filename = filename.as_ref().to_string();
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

    pub async fn open(self, parent: Option<&Window>) -> io::Result<Option<PathBuf>> {
        unsafe {
            filebox(parent, self.title, self.filename, self.filters, true, false)
                .await?
                .result()
        }
    }

    pub async fn open_multiple(self, parent: Option<&Window>) -> io::Result<Vec<PathBuf>> {
        unsafe {
            filebox(parent, self.title, self.filename, self.filters, true, true)
                .await?
                .results()
        }
    }

    pub async fn save(self, parent: Option<&Window>) -> io::Result<Option<PathBuf>> {
        unsafe {
            filebox(
                parent,
                self.title,
                self.filename,
                self.filters,
                false,
                false,
            )
            .await?
            .result()
        }
    }
}

async unsafe fn filebox(
    parent: Option<&Window>,
    title: String,
    filename: String,
    filters: Vec<FileFilter>,
    open: bool,
    multiple: bool,
) -> io::Result<FileBoxInner> {
    let mtm = MainThreadMarker::new().unwrap();
    let handle: Id<NSSavePanel> = if open {
        let handle = NSOpenPanel::openPanel(mtm);
        handle.setCanChooseFiles(true);
        handle.setCanChooseDirectories(false);
        handle.setResolvesAliases(false);
        if multiple {
            handle.setAllowsMultipleSelection(true);
        }
        Id::into_super(handle)
    } else {
        let handle = NSSavePanel::savePanel(mtm);
        handle.setCanCreateDirectories(true);
        handle
    };
    handle.setShowsHiddenFiles(true);
    handle.setExtensionHidden(false);
    handle.setCanSelectHiddenExtension(false);
    handle.setTreatsFilePackagesAsDirectories(true);

    if let Some(parent) = parent {
        handle.setParentWindow(Some(&parent.as_nswindow()));
    }

    if !title.is_empty() {
        handle.setTitle(Some(&NSString::from_str(&title)));
    }

    handle.setNameFieldStringValue(&NSString::from_str(&filename));
    if !filters.is_empty() {
        let allow_others = filters
            .iter()
            .any(|f| f.pattern() == "*.*" || f.pattern() == "*");
        handle.setAllowsOtherFileTypes(allow_others);

        let ns_filters = NSArray::from_vec(
            filters
                .into_iter()
                .filter_map(|f| {
                    let pattern = f.pattern;
                    if pattern == "*.*" || pattern == "*" {
                        None
                    } else {
                        Some(NSString::from_str(
                            pattern.strip_prefix("*.").unwrap_or(&pattern),
                        ))
                    }
                })
                .collect::<Vec<_>>(),
        );
        if !ns_filters.is_empty() {
            #[allow(deprecated)]
            handle.setAllowedFileTypes(Some(&ns_filters));
        }
    }

    let res = if let Some(parent) = parent {
        let (tx, rx) = futures_channel::oneshot::channel();
        let tx = RefCell::new(Some(tx));
        let block = ConcreteBlock::new(|res| {
            tx.borrow_mut()
                .take()
                .expect("the handler should only be called once")
                .send(res)
                .ok();
        });
        handle.beginSheetModalForWindow_completionHandler(&parent.as_nswindow(), &block);
        rx.await.expect("NSAlert cancelled")
    } else {
        handle.runModal()
    };
    Ok(FileBoxInner(if res == NSModalResponseOK {
        Some(handle)
    } else {
        None
    }))
}

struct FileBoxInner(Option<Id<NSSavePanel>>);

impl FileBoxInner {
    pub unsafe fn result(self) -> io::Result<Option<PathBuf>> {
        if let Some(dialog) = self.0 {
            Ok(dialog
                .URL()
                .and_then(|url| url.path())
                .map(|s| PathBuf::from(from_nsstring(&s))))
        } else {
            Ok(None)
        }
    }

    pub unsafe fn results(self) -> io::Result<Vec<PathBuf>> {
        if let Some(dialog) = self.0 {
            let dialog: Id<NSOpenPanel> = Id::cast(dialog);
            Ok(dialog
                .URLs()
                .into_iter()
                .filter_map(|url| url.path().map(|s| PathBuf::from(from_nsstring(&s))))
                .collect())
        } else {
            Ok(vec![])
        }
    }
}
