use std::{mem::ManuallyDrop, path::PathBuf, ptr::null_mut};

use cxx::{ExternType, type_id};
use futures_channel::oneshot;

use crate::{AsRawWindow, AsWindow};

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
    title: String,
    filename: String,
    filters: Vec<FileFilter>,
}

impl FileBox {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn title(&mut self, title: &str) {
        self.title = title.to_string();
    }

    pub fn filename(&mut self, filename: &str) {
        self.filename = filename.to_string();
    }

    pub fn filters(&mut self, filters: impl IntoIterator<Item = FileFilter>) {
        self.filters = filters.into_iter().collect();
    }

    pub fn add_filter(&mut self, filter: FileFilter) {
        self.filters.push(filter);
    }

    pub async fn open(self, parent: Option<impl AsWindow>) -> Option<PathBuf> {
        self.filebox(parent, true, false, false)
            .await
            .into_iter()
            .next()
    }

    pub async fn open_multiple(self, parent: Option<impl AsWindow>) -> Vec<PathBuf> {
        self.filebox(parent, true, true, false).await
    }

    pub async fn open_folder(self, parent: Option<impl AsWindow>) -> Option<PathBuf> {
        self.filebox(parent, true, false, true)
            .await
            .into_iter()
            .next()
    }

    pub async fn save(self, parent: Option<impl AsWindow>) -> Option<PathBuf> {
        self.filebox(parent, false, false, false)
            .await
            .into_iter()
            .next()
    }

    async fn filebox(
        self,
        parent: Option<impl AsWindow>,
        open: bool,
        multiple: bool,
        folder: bool,
    ) -> Vec<PathBuf> {
        let parent = parent
            .map(|p| p.as_window().as_raw_window())
            .unwrap_or(null_mut());
        let mut b = unsafe { ffi::new_file_dialog(parent) };

        if open {
            b.pin_mut().setAcceptMode(QFileDialogAcceptMode::AcceptOpen);
            b.pin_mut().setFileMode(if multiple {
                QFileDialogFileMode::ExistingFiles
            } else if folder {
                QFileDialogFileMode::Directory
            } else {
                QFileDialogFileMode::ExistingFile
            });
        } else {
            b.pin_mut().setAcceptMode(QFileDialogAcceptMode::AcceptSave);
            b.pin_mut().setFileMode(QFileDialogFileMode::AnyFile);
        }

        let filter = self
            .filters
            .iter()
            .map(|f| format!("{}({})", f.name, f.pattern))
            .collect::<Vec<_>>()
            .join(";;");
        ffi::file_dialog_set_texts(b.pin_mut(), &self.title, &self.filename, &filter);

        let (tx, rx) = oneshot::channel::<i32>();
        let tx = ManuallyDrop::new(Some(tx));
        unsafe {
            ffi::file_dialog_connect_finished(
                b.pin_mut(),
                dialog_finished,
                std::ptr::addr_of!(tx).cast(),
            );
        }
        b.pin_mut().open();
        let res = rx.await.unwrap();
        if res == 0 {
            return vec![];
        }

        ffi::file_dialog_files(&b)
            .into_iter()
            .map(PathBuf::from)
            .collect()
    }
}

fn dialog_finished(data: *const u8, res: i32) {
    if let Some(tx) = unsafe { (data.cast_mut() as *mut Option<oneshot::Sender<i32>>).as_mut() } {
        if let Some(tx) = tx.take() {
            tx.send(res).ok();
        }
    }
}

#[repr(i32)]
enum QFileDialogAcceptMode {
    AcceptOpen,
    AcceptSave,
}

unsafe impl ExternType for QFileDialogAcceptMode {
    type Id = type_id!("QFileDialogAcceptMode");
    type Kind = cxx::kind::Trivial;
}

#[repr(i32)]
enum QFileDialogFileMode {
    AnyFile,
    ExistingFile,
    Directory,
    ExistingFiles,
}

unsafe impl ExternType for QFileDialogFileMode {
    type Id = type_id!("QFileDialogFileMode");
    type Kind = cxx::kind::Trivial;
}

#[cxx::bridge]
mod ffi {
    unsafe extern "C++" {
        include!("winio/src/ui/qt/filebox.hpp");

        type QFileDialog;
        type QWidget = crate::ui::QWidget;
        type QFileDialogAcceptMode = super::QFileDialogAcceptMode;
        type QFileDialogFileMode = super::QFileDialogFileMode;

        fn setAcceptMode(self: Pin<&mut QFileDialog>, mode: QFileDialogAcceptMode);
        fn setFileMode(self: Pin<&mut QFileDialog>, mode: QFileDialogFileMode);
        fn open(self: Pin<&mut QFileDialog>);

        unsafe fn new_file_dialog(parent: *mut QWidget) -> UniquePtr<QFileDialog>;
        unsafe fn file_dialog_connect_finished(
            b: Pin<&mut QFileDialog>,
            callback: unsafe fn(*const u8, i32),
            data: *const u8,
        );
        fn file_dialog_set_texts(
            b: Pin<&mut QFileDialog>,
            title: &str,
            filename: &str,
            filter: &str,
        );
        fn file_dialog_files(b: &QFileDialog) -> Vec<String>;
    }
}
