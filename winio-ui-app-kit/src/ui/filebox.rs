use std::{cell::Cell, path::PathBuf, rc::Rc};

use block2::StackBlock;
use objc2::{MainThreadOnly, rc::Retained};
use objc2_app_kit::{NSModalResponseOK, NSOpenPanel, NSSavePanel};
use objc2_foundation::{MainThreadMarker, NSArray, NSString};
use objc2_uniform_type_identifiers::UTType;
use winio_handle::{AsRawWindow, AsWindow};

use crate::{Error, Result, catch, ui::from_nsstring};

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

// SAFETY: NSString is thread-safe.
unsafe impl Send for FileBox {}
unsafe impl Sync for FileBox {}

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

    pub async fn open(self, parent: Option<impl AsWindow>) -> Result<Option<PathBuf>> {
        filebox(
            parent,
            self.title,
            self.filename,
            self.filters,
            true,
            false,
            false,
        )
        .await?
        .result()
    }

    pub async fn open_multiple(self, parent: Option<impl AsWindow>) -> Result<Vec<PathBuf>> {
        filebox(
            parent,
            self.title,
            self.filename,
            self.filters,
            true,
            true,
            false,
        )
        .await?
        .results()
    }

    pub async fn open_folder(self, parent: Option<impl AsWindow>) -> Result<Option<PathBuf>> {
        filebox(
            parent,
            self.title,
            self.filename,
            self.filters,
            true,
            false,
            true,
        )
        .await?
        .result()
    }

    pub async fn save(self, parent: Option<impl AsWindow>) -> Result<Option<PathBuf>> {
        filebox(
            parent,
            self.title,
            self.filename,
            self.filters,
            false,
            false,
            false,
        )
        .await?
        .result()
    }
}

async fn filebox(
    parent: Option<impl AsWindow>,
    title: Retained<NSString>,
    filename: Retained<NSString>,
    filters: Vec<FileFilter>,
    open: bool,
    multiple: bool,
    folder: bool,
) -> Result<FileBoxInner> {
    let parent = parent.map(|p| p.as_window().as_raw_window());
    let mtm = parent
        .as_ref()
        .map(|w| w.mtm())
        .or_else(MainThreadMarker::new)
        .ok_or(Error::NotMainThread)?;

    let handle = catch(|| {
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
            unsafe { handle.setParentWindow(Some(parent)) };
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

            if !(open && allow_others) {
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
        }
        handle
    })?;

    let res = if let Some(parent) = &parent {
        let (tx, rx) = local_sync::oneshot::channel();
        let tx = Rc::new(Cell::new(Some(tx)));
        let block = StackBlock::new(move |res| {
            tx.take()
                .expect("the handler should only be called once")
                .send(res)
                .ok();
        });
        catch(|| handle.beginSheetModalForWindow_completionHandler(parent, &block))?;
        rx.await?
    } else {
        catch(|| handle.runModal())?
    };
    handle.close();
    Ok(FileBoxInner(if res == NSModalResponseOK {
        Some(handle)
    } else {
        None
    }))
}

struct FileBoxInner(Option<Retained<NSSavePanel>>);

impl FileBoxInner {
    pub fn result(self) -> Result<Option<PathBuf>> {
        if let Some(dialog) = self.0 {
            catch(|| {
                dialog
                    .URL()
                    .and_then(|url| url.path())
                    .map(|s| PathBuf::from(from_nsstring(&s)))
            })
        } else {
            Ok(None)
        }
    }

    pub fn results(self) -> Result<Vec<PathBuf>> {
        if let Some(dialog) = self.0 {
            let dialog: Retained<NSOpenPanel> = unsafe { Retained::cast_unchecked(dialog) };
            catch(|| {
                dialog
                    .URLs()
                    .iter()
                    .filter_map(|url| url.path().map(|s| PathBuf::from(from_nsstring(&s))))
                    .collect()
            })
        } else {
            Ok(vec![])
        }
    }
}
