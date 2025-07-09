use std::path::PathBuf;

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

pub fn filebox(
    parent: Option<HWND>,
    title: U16CString,
    filename: U16CString,
    filters: Vec<FileFilter>,
    open: bool,
    multiple: bool,
    folder: bool,
) -> FileBoxInner {
    let init = CoInitialize::init();

    unsafe {
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
}

pub struct FileBoxInner(Option<IFileDialog>, CoInitialize);

impl FileBoxInner {
    pub fn result(self) -> Option<PathBuf> {
        if let Some(dialog) = self.0 {
            unsafe {
                let item = dialog.GetResult().unwrap();
                let name_ptr = item.GetDisplayName(SIGDN_FILESYSPATH).unwrap();
                let name_ptr = CoTaskMemPtr(name_ptr.0);
                let name = U16CStr::from_ptr_str(name_ptr.0).to_os_string();
                Some(PathBuf::from(name))
            }
        } else {
            None
        }
    }

    pub fn results(self) -> Vec<PathBuf> {
        if let Some(dialog) = self.0 {
            unsafe {
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
            }
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
