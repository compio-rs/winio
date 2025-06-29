use std::{panic::resume_unwind, path::PathBuf};

use widestring::{U16CStr, U16CString};
use windows::{
    Win32::{
        Foundation::{ERROR_CANCELLED, HWND},
        System::Com::{
            CLSCTX_INPROC_SERVER, COINIT_APARTMENTTHREADED, CoCreateInstance, CoInitializeEx,
            CoTaskMemFree, CoUninitialize,
        },
        UI::Shell::{
            Common::COMDLG_FILTERSPEC, FOS_ALLOWMULTISELECT, FOS_PICKFOLDERS, FileOpenDialog,
            FileSaveDialog, IFileDialog, IFileOpenDialog, SIGDN_FILESYSPATH,
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
}

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
        let parent = parent.map(|p| p.as_window().as_raw_window() as isize);
        compio::runtime::spawn_blocking(move || unsafe {
            let parent = parent.map(|w| HWND(w as _));
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
        let parent = parent.map(|p| p.as_window().as_raw_window() as isize);
        compio::runtime::spawn_blocking(move || unsafe {
            let parent = parent.map(|w| HWND(w as _));
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
        let parent = parent.map(|p| p.as_window().as_raw_window() as isize);
        compio::runtime::spawn_blocking(move || unsafe {
            let parent = parent.map(|w| HWND(w as _));
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
        let parent = parent.map(|p| p.as_window().as_raw_window() as isize);
        compio::runtime::spawn_blocking(move || unsafe {
            let parent = parent.map(|w| HWND(w as _));
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

unsafe fn filebox(
    parent: Option<HWND>,
    title: U16CString,
    filename: U16CString,
    filters: Vec<FileFilter>,
    open: bool,
    multiple: bool,
    folder: bool,
) -> FileBoxInner {
    let init = CoInitialize::init();

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

    if folder {
        debug_assert!(open, "Cannot save to a folder.");

        let mut opts = handle.GetOptions().unwrap();
        opts |= FOS_PICKFOLDERS;
        handle.SetOptions(opts).unwrap();
    }

    let handle = match handle.Show(parent) {
        Ok(()) => Some(handle),
        Err(e) if e.code() == HRESULT::from(ERROR_CANCELLED) => None,
        Err(e) => panic!("{e:?}"),
    };

    FileBoxInner(handle, init)
}

struct FileBoxInner(Option<IFileDialog>, CoInitialize);

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

struct CoInitialize;

impl CoInitialize {
    pub fn init() -> Self {
        unsafe {
            CoInitializeEx(None, COINIT_APARTMENTTHREADED).unwrap();
        }
        Self
    }
}

impl Drop for CoInitialize {
    fn drop(&mut self) {
        unsafe { CoUninitialize() };
    }
}
