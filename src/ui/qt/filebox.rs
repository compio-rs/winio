use std::{io, mem::ManuallyDrop, path::PathBuf};

use cxx::{ExternType, type_id};
use futures_channel::oneshot;

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
        self.filebox(parent, true, false)
            .await
            .map(|files| files.into_iter().next())
    }

    pub async fn open_multiple(self, parent: Option<&Window>) -> io::Result<Vec<PathBuf>> {
        self.filebox(parent, true, true).await
    }

    pub async fn save(self, parent: Option<&Window>) -> io::Result<Option<PathBuf>> {
        self.filebox(parent, false, false)
            .await
            .map(|files| files.into_iter().next())
    }

    async fn filebox(
        self,
        parent: Option<&Window>,
        open: bool,
        multiple: bool,
    ) -> io::Result<Vec<PathBuf>> {
        let mut b = if let Some(parent) = parent {
            parent.pin_mut(ffi::new_file_dialog_with_parent)
        } else {
            ffi::new_file_dialog()
        };

        if open {
            b.pin_mut().setAcceptMode(QFileDialogAcceptMode::AcceptOpen);
            b.pin_mut().setFileMode(if multiple {
                QFileDialogFileMode::ExistingFiles
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
        let res = rx
            .await
            .map_err(|_| io::Error::new(io::ErrorKind::Other, "cannot receive result"))?;
        if res == 0 {
            return Ok(vec![]);
        }

        Ok(ffi::file_dialog_files(&b)
            .into_iter()
            .map(PathBuf::from)
            .collect())
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
#[allow(dead_code)]
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
        type QWidget = crate::QWidget;
        type QFileDialogAcceptMode = super::QFileDialogAcceptMode;
        type QFileDialogFileMode = super::QFileDialogFileMode;

        fn setAcceptMode(self: Pin<&mut QFileDialog>, mode: QFileDialogAcceptMode);
        fn setFileMode(self: Pin<&mut QFileDialog>, mode: QFileDialogFileMode);
        fn open(self: Pin<&mut QFileDialog>);

        fn new_file_dialog() -> UniquePtr<QFileDialog>;
        #[rust_name = "new_file_dialog_with_parent"]
        fn new_file_dialog(parent: Pin<&mut QWidget>) -> UniquePtr<QFileDialog>;
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
