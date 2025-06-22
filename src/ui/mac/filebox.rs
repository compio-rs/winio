use std::{cell::RefCell, path::PathBuf, rc::Rc};

use block2::StackBlock;
use objc2::rc::Retained;
use objc2_app_kit::{NSModalResponseOK, NSOpenPanel, NSSavePanel};
use objc2_foundation::{MainThreadMarker, NSArray, NSString};
use objc2_uniform_type_identifiers::UTType;

use crate::{AsRawWindow, AsWindow, ui::from_nsstring};

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

    pub async fn open(self, parent: Option<impl AsWindow>) -> Option<PathBuf> {
        unsafe {
            filebox(
                parent,
                self.title,
                self.filename,
                self.filters,
                true,
                false,
                false,
            )
            .await
            .result()
        }
    }

    pub async fn open_multiple(self, parent: Option<impl AsWindow>) -> Vec<PathBuf> {
        unsafe {
            filebox(
                parent,
                self.title,
                self.filename,
                self.filters,
                true,
                true,
                false,
            )
            .await
            .results()
        }
    }

    pub async fn open_folder(self, parent: Option<impl AsWindow>) -> Option<PathBuf> {
        unsafe {
            filebox(
                parent,
                self.title,
                self.filename,
                self.filters,
                true,
                false,
                true,
            )
            .await
            .result()
        }
    }

    pub async fn save(self, parent: Option<impl AsWindow>) -> Option<PathBuf> {
        unsafe {
            filebox(
                parent,
                self.title,
                self.filename,
                self.filters,
                false,
                false,
                false,
            )
            .await
            .result()
        }
    }
}

async unsafe fn filebox(
    parent: Option<impl AsWindow>,
    title: Retained<NSString>,
    filename: Retained<NSString>,
    filters: Vec<FileFilter>,
    open: bool,
    multiple: bool,
    folder: bool,
) -> FileBoxInner {
    let parent = parent.map(|p| p.as_window().as_raw_window());

    let mtm = MainThreadMarker::new().unwrap();
    let handle: Retained<NSSavePanel> = if open {
        let handle = NSOpenPanel::openPanel(mtm);
        handle.setCanChooseFiles(!folder);
        handle.setCanChooseDirectories(folder);
        handle.setResolvesAliases(false);
        if multiple {
            handle.setAllowsMultipleSelection(true);
        }
        Retained::into_super(handle)
    } else {
        let handle = NSSavePanel::savePanel(mtm);
        handle.setCanCreateDirectories(true);
        handle
    };
    handle.setShowsHiddenFiles(true);
    handle.setExtensionHidden(false);
    handle.setCanSelectHiddenExtension(false);
    handle.setTreatsFilePackagesAsDirectories(true);

    if let Some(parent) = &parent {
        handle.setParentWindow(Some(parent));
    }

    if !title.is_empty() {
        handle.setTitle(Some(&title));
    }

    handle.setNameFieldStringValue(&filename);
    if !filters.is_empty() {
        let allow_others = filters
            .iter()
            .any(|f| f.pattern == "*.*" || f.pattern == "*");
        handle.setAllowsOtherFileTypes(allow_others);

        let ns_filters = NSArray::from_retained_slice(
            &filters
                .into_iter()
                .filter_map(|f| {
                    let pattern = f.pattern;
                    if pattern == "*.*" || pattern == "*" {
                        None
                    } else {
                        UTType::typeWithFilenameExtension(&NSString::from_str(
                            pattern.strip_prefix("*.").unwrap_or(&pattern),
                        ))
                    }
                })
                .collect::<Vec<_>>(),
        );
        if !ns_filters.is_empty() {
            handle.setAllowedContentTypes(&ns_filters);
        }
    }

    let res = if let Some(parent) = &parent {
        let (tx, rx) = futures_channel::oneshot::channel();
        let tx = Rc::new(RefCell::new(Some(tx)));
        let block = StackBlock::new(move |res| {
            tx.borrow_mut()
                .take()
                .expect("the handler should only be called once")
                .send(res)
                .ok();
        });
        handle.beginSheetModalForWindow_completionHandler(parent, &block);
        rx.await.expect("NSAlert cancelled")
    } else {
        handle.runModal()
    };
    handle.close();
    FileBoxInner(if res == NSModalResponseOK {
        Some(handle)
    } else {
        None
    })
}

struct FileBoxInner(Option<Retained<NSSavePanel>>);

impl FileBoxInner {
    pub unsafe fn result(self) -> Option<PathBuf> {
        if let Some(dialog) = self.0 {
            dialog
                .URL()
                .and_then(|url| url.path())
                .map(|s| PathBuf::from(from_nsstring(&s)))
        } else {
            None
        }
    }

    pub unsafe fn results(self) -> Vec<PathBuf> {
        if let Some(dialog) = self.0 {
            let dialog: Retained<NSOpenPanel> = Retained::cast_unchecked(dialog);
            dialog
                .URLs()
                .iter()
                .filter_map(|url| url.path().map(|s| PathBuf::from(from_nsstring(&s))))
                .collect()
        } else {
            vec![]
        }
    }
}
