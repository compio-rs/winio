use std::{panic::resume_unwind, path::PathBuf};

use widestring::{U16CStr, U16CString};
use windows::{
    Win32::{
        Foundation::{ERROR_CANCELLED, HWND},
        System::Com::{CLSCTX_INPROC_SERVER, CoCreateInstance, CoTaskMemFree},
        UI::Shell::{
            Common::COMDLG_FILTERSPEC, FOS_ALLOWMULTISELECT, FileOpenDialog, FileSaveDialog,
            IFileDialog, IFileOpenDialog, SIGDN_FILESYSPATH,
        },
    },
    core::{HRESULT, Interface, PCWSTR},
};

use crate::{AsRawWindow, AsWindow};

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

    pub async fn open(self, parent: Option<impl AsWindow>) -> Option<PathBuf> {
        let parent = parent
            .map(|p| p.as_window().as_raw_window() as isize)
            .unwrap_or_default();
        compio::runtime::spawn_blocking(move || unsafe {
            let parent = HWND(parent as _);
            filebox(parent, self.title, self.filename, self.filters, true, false).result()
        })
        .await
        .unwrap_or_else(|e| resume_unwind(e))
    }

    pub async fn open_multiple(self, parent: Option<impl AsWindow>) -> Vec<PathBuf> {
        let parent = parent
            .map(|p| p.as_window().as_raw_window() as isize)
            .unwrap_or_default();
        compio::runtime::spawn_blocking(move || unsafe {
            let parent = HWND(parent as _);
            filebox(parent, self.title, self.filename, self.filters, true, true).results()
        })
        .await
        .unwrap_or_else(|e| resume_unwind(e))
    }

    pub async fn save(self, parent: Option<impl AsWindow>) -> Option<PathBuf> {
        let parent = parent
            .map(|p| p.as_window().as_raw_window() as isize)
            .unwrap_or_default();
        compio::runtime::spawn_blocking(move || unsafe {
            let parent = HWND(parent as _);
            filebox(
                parent,
                self.title,
                self.filename,
                self.filters,
                false,
                false,
            )
            .result()
        })
        .await
        .unwrap_or_else(|e| resume_unwind(e))
    }
}

unsafe fn filebox(
    parent: HWND,
    title: U16CString,
    filename: U16CString,
    filters: Vec<FileFilter>,
    open: bool,
    multiple: bool,
) -> FileBoxInner {
    let handle: IFileDialog = if open {
        CoCreateInstance(&FileOpenDialog, None, CLSCTX_INPROC_SERVER).unwrap()
    } else {
        CoCreateInstance(&FileSaveDialog, None, CLSCTX_INPROC_SERVER).unwrap()
    };

    if !title.is_empty() {
        handle.SetTitle(PCWSTR(title.as_ptr())).unwrap();
    }
    if !filename.is_empty() {
        handle.SetFileName(PCWSTR(filename.as_ptr())).unwrap();
    }

    let types = filters
        .iter()
        .map(|filter| COMDLG_FILTERSPEC {
            pszName: PCWSTR(filter.name.as_ptr()),
            pszSpec: PCWSTR(filter.pattern.as_ptr()),
        })
        .collect::<Vec<_>>();
    handle.SetFileTypes(&types).unwrap();

    if multiple {
        debug_assert!(open, "Cannot save to multiple targets.");

        let mut opts = handle.GetOptions().unwrap();
        opts |= FOS_ALLOWMULTISELECT;
        handle.SetOptions(opts).unwrap();
    }

    let handle = match handle.Show(parent) {
        Ok(()) => Some(handle),
        Err(e) if e.code() == HRESULT::from(ERROR_CANCELLED) => None,
        Err(e) => panic!("{:?}", e),
    };

    FileBoxInner(handle)
}

struct FileBoxInner(Option<IFileDialog>);

impl FileBoxInner {
    pub unsafe fn result(self) -> Option<PathBuf> {
        if let Some(dialog) = self.0 {
            let item = dialog.GetResult().unwrap();
            let name_ptr = item.GetDisplayName(SIGDN_FILESYSPATH).unwrap();
            let name_ptr = CoTaskMemPtr(name_ptr.0);
            let name = U16CStr::from_ptr_str(name_ptr.0).to_os_string();
            Some(PathBuf::from(name))
        } else {
            None
        }
    }

    pub unsafe fn results(self) -> Vec<PathBuf> {
        if let Some(dialog) = self.0 {
            let handle: IFileOpenDialog = dialog.cast().unwrap();
            let results = handle.GetResults().unwrap();
            let count = results.GetCount().unwrap();
            let mut names = vec![];
            for i in 0..count {
                let item = results.GetItemAt(i).unwrap();
                let name_ptr = item.GetDisplayName(SIGDN_FILESYSPATH).unwrap();
                let name_ptr = CoTaskMemPtr(name_ptr.0);
                let name = U16CStr::from_ptr_str(name_ptr.0).to_os_string();
                names.push(PathBuf::from(name));
            }
            names
        } else {
            vec![]
        }
    }
}

struct CoTaskMemPtr<T>(*mut T);

impl<T> Drop for CoTaskMemPtr<T> {
    fn drop(&mut self) {
        unsafe { CoTaskMemFree(Some(self.0.cast())) }
    }
}
