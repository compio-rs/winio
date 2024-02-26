use std::{io, path::PathBuf};

use widestring::{U16CStr, U16CString};
use windows::{
    core::{Interface, HRESULT, PCWSTR},
    Win32::{
        Foundation::{ERROR_CANCELLED, HWND},
        System::Com::{CoCreateInstance, CoTaskMemFree, CLSCTX_INPROC_SERVER},
        UI::Shell::{
            Common::COMDLG_FILTERSPEC, FileOpenDialog, FileSaveDialog, IFileDialog,
            IFileOpenDialog, FOS_ALLOWMULTISELECT, SIGDN_FILESYSPATH,
        },
    },
};

use crate::ui::{AsRawWindow, Window};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileFilter {
    name: U16CString,
    pattern: U16CString,
}

impl FileFilter {
    pub fn new(name: &str, pattern: &str) -> Self {
        Self {
            name: U16CString::from_str_truncate(name),
            pattern: U16CString::from_str_truncate(pattern),
        }
    }

    pub fn name(&self) -> &U16CStr {
        &self.name
    }

    pub fn pattern(&self) -> &U16CStr {
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
    title: U16CString,
    filename: U16CString,
    filters: Vec<FileFilter>,
}

impl FileBox {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn title(mut self, title: impl AsRef<str>) -> Self {
        self.title = U16CString::from_str_truncate(title);
        self
    }

    pub fn filename(mut self, filename: impl AsRef<str>) -> Self {
        self.filename = U16CString::from_str_truncate(filename);
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
        let parent = HWND(parent.map(|p| p.as_raw_window()).unwrap_or_default());
        compio::runtime::spawn_blocking(move || unsafe {
            filebox(parent, self.title, self.filename, self.filters, true, false)?.result()
        })
        .await
    }

    pub async fn open_multiple(self, parent: Option<&Window>) -> io::Result<Vec<PathBuf>> {
        let parent = HWND(parent.map(|p| p.as_raw_window()).unwrap_or_default());
        compio::runtime::spawn_blocking(move || unsafe {
            filebox(parent, self.title, self.filename, self.filters, true, true)?.results()
        })
        .await
    }

    pub async fn save(self, parent: Option<&Window>) -> io::Result<Option<PathBuf>> {
        let parent = HWND(parent.map(|p| p.as_raw_window()).unwrap_or_default());
        compio::runtime::spawn_blocking(move || unsafe {
            filebox(
                parent,
                self.title,
                self.filename,
                self.filters,
                false,
                false,
            )?
            .result()
        })
        .await
    }
}

unsafe fn filebox(
    parent: HWND,
    title: U16CString,
    filename: U16CString,
    filters: Vec<FileFilter>,
    open: bool,
    multiple: bool,
) -> io::Result<FileBoxInner> {
    let handle: IFileDialog = if open {
        CoCreateInstance(&FileOpenDialog, None, CLSCTX_INPROC_SERVER)?
    } else {
        CoCreateInstance(&FileSaveDialog, None, CLSCTX_INPROC_SERVER)?
    };

    if !title.is_empty() {
        handle.SetTitle(PCWSTR(title.as_ptr()))?;
    }
    if !filename.is_empty() {
        handle.SetFileName(PCWSTR(filename.as_ptr()))?;
    }

    let types = filters
        .iter()
        .map(|filter| COMDLG_FILTERSPEC {
            pszName: PCWSTR(filter.name.as_ptr()),
            pszSpec: PCWSTR(filter.pattern.as_ptr()),
        })
        .collect::<Vec<_>>();
    handle.SetFileTypes(&types)?;

    if multiple {
        debug_assert!(open, "Cannot save to multiple targets.");

        let mut opts = handle.GetOptions()?;
        opts |= FOS_ALLOWMULTISELECT;
        handle.SetOptions(opts)?;
    }

    let handle = match handle.Show(parent) {
        Ok(()) => Some(handle),
        Err(e) if e.code() == HRESULT::from(ERROR_CANCELLED) => None,
        Err(e) => return Err(e.into()),
    };

    Ok(FileBoxInner(handle))
}

struct FileBoxInner(Option<IFileDialog>);

impl FileBoxInner {
    pub unsafe fn result(self) -> io::Result<Option<PathBuf>> {
        if let Some(dialog) = self.0 {
            let item = dialog.GetResult()?;
            let name_ptr = item.GetDisplayName(SIGDN_FILESYSPATH)?;
            let name_ptr = CoTaskMemPtr(name_ptr.0);
            let name = U16CStr::from_ptr_str(name_ptr.0).to_os_string();
            Ok(Some(PathBuf::from(name)))
        } else {
            Ok(None)
        }
    }

    pub unsafe fn results(self) -> io::Result<Vec<PathBuf>> {
        if let Some(dialog) = self.0 {
            let handle: IFileOpenDialog = dialog.cast()?;
            let results = handle.GetResults()?;
            let count = results.GetCount()?;
            let mut names = vec![];
            for i in 0..count {
                let item = results.GetItemAt(i)?;
                let name_ptr = item.GetDisplayName(SIGDN_FILESYSPATH)?;
                let name_ptr = CoTaskMemPtr(name_ptr.0);
                let name = U16CStr::from_ptr_str(name_ptr.0).to_os_string();
                names.push(PathBuf::from(name));
            }
            Ok(names)
        } else {
            Ok(vec![])
        }
    }
}

struct CoTaskMemPtr<T>(*mut T);

impl<T> Drop for CoTaskMemPtr<T> {
    fn drop(&mut self) {
        unsafe { CoTaskMemFree(Some(self.0.cast())) }
    }
}
