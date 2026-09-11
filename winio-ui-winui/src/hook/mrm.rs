//! Hook MRM.dll to avoid errors
//! <https://github.com/microsoft/WindowsAppSDK/issues/5814>

use std::{env::current_exe, sync::Once};

use compio_log::error;
use slim_detours_sys::SlimDetoursInlineHook;
use sync_unsafe_cell::SyncUnsafeCell;
use windows::Win32::{
    Foundation::{E_NOTIMPL, ERROR_FILE_NOT_FOUND, S_OK},
    System::{
        Com::CoTaskMemAlloc,
        LibraryLoader::{GetProcAddress, LoadLibraryW},
    },
};
use windows_core::{HRESULT, HSTRING, PCWSTR, PWSTR, Result, s, w};

type MrmGetFilePathFromNameFn =
    unsafe extern "system" fn(filename: PCWSTR, filepath: *mut PWSTR) -> HRESULT;
static TRUE_MRM_GET_FILE_PATH_FROM_NAME: SyncUnsafeCell<Option<MrmGetFilePathFromNameFn>> =
    SyncUnsafeCell::new(None);

fn get_resource_filename() -> Option<PWSTR> {
    let mut exe_path = current_exe().ok()?;
    exe_path.set_file_name("resources.pri");
    let path = HSTRING::from(exe_path.as_path());
    unsafe {
        let ptr = CoTaskMemAlloc((path.len() + 1) * 2);
        if ptr.is_null() {
            return None;
        }
        let slice: &mut [u16] = std::slice::from_raw_parts_mut(ptr.cast(), path.len() + 1);
        slice.get_unchecked_mut(..path.len()).copy_from_slice(&path);
        *slice.get_unchecked_mut(path.len()) = 0;
        Some(PWSTR(ptr.cast()))
    }
}

unsafe extern "system" fn mrm_get_file_path_from_name(
    filename: PCWSTR,
    filepath: *mut PWSTR,
) -> HRESULT {
    match unsafe { *TRUE_MRM_GET_FILE_PATH_FROM_NAME.get() } {
        Some(f) => {
            let mut res = unsafe { f(filename, filepath) };
            if res == ERROR_FILE_NOT_FOUND.to_hresult()
                && let Some(ptr) = get_resource_filename()
                && let Some(filepath) = unsafe { filepath.as_mut() }
            {
                *filepath = ptr;
                res = S_OK;
            }
            res
        }
        None => E_NOTIMPL,
    }
}

#[allow(clippy::missing_transmute_annotations)]
fn detour_attach() -> Result<()> {
    unsafe {
        let module = LoadLibraryW(w!("MRM.dll"));
        if module.0.is_null() {
            return Err(windows_core::Error::from_thread());
        }
        let func = GetProcAddress(module, s!("MrmGetFilePathFromName"));
        *TRUE_MRM_GET_FILE_PATH_FROM_NAME.get() = std::mem::transmute(func);

        let res = SlimDetoursInlineHook(
            1,
            TRUE_MRM_GET_FILE_PATH_FROM_NAME.get().cast(),
            mrm_get_file_path_from_name as _,
        );
        HRESULT(res).ok()
    }
}

static DETOUR_GUARD: Once = Once::new();

pub fn init_hook() {
    DETOUR_GUARD.call_once(|| {
        if let Err(_e) = detour_attach() {
            error!("Failed to hook MRM.dll: {_e:?}");
        }
    });
}
