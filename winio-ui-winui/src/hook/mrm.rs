//! Hook MRM.dll to avoid errors
//! https://github.com/microsoft/WindowsAppSDK/issues/5814

use std::{env::current_exe, sync::Once};

use slim_detours_sys::SlimDetoursInlineHook;
use sync_unsafe_cell::SyncUnsafeCell;
use windows::{
    Win32::System::{
        Com::CoTaskMemAlloc,
        LibraryLoader::{GetProcAddress, LoadLibraryW},
    },
    core::{HSTRING, Result, s, w},
};
use windows_sys::{
    Win32::{
        Foundation::{E_NOTIMPL, ERROR_FILE_NOT_FOUND, S_OK},
        System::Diagnostics::Debug::FACILITY_WIN32,
    },
    core::{HRESULT, PCWSTR, PWSTR},
};

const fn hresult_from_win32(x: i32) -> HRESULT {
    if x <= 0 {
        x
    } else {
        ((x) & 0x0000FFFF) | ((FACILITY_WIN32 as HRESULT) << 16) | 0x80000000u32 as HRESULT
    }
}

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
        Some(ptr.cast())
    }
}

unsafe extern "system" fn mrm_get_file_path_from_name(
    filename: PCWSTR,
    filepath: *mut PWSTR,
) -> HRESULT {
    match *TRUE_MRM_GET_FILE_PATH_FROM_NAME.get() {
        Some(f) => {
            let mut res = f(filename, filepath);
            if res == hresult_from_win32(ERROR_FILE_NOT_FOUND as _) {
                if let Some(ptr) = get_resource_filename() {
                    *filepath = ptr;
                    res = S_OK;
                }
            }
            res
        }
        None => E_NOTIMPL,
    }
}

#[allow(clippy::missing_transmute_annotations)]
fn detour_attach() -> Result<()> {
    unsafe {
        let module = LoadLibraryW(w!("MRM.dll"))?;
        let func = GetProcAddress(module, s!("MrmGetFilePathFromName"));
        *TRUE_MRM_GET_FILE_PATH_FROM_NAME.get() = std::mem::transmute(func);

        let res = SlimDetoursInlineHook(
            1,
            TRUE_MRM_GET_FILE_PATH_FROM_NAME.get().cast(),
            mrm_get_file_path_from_name as _,
        );
        windows::core::HRESULT(res).ok()
    }
}

static DETOUR_GUARD: Once = Once::new();

pub fn init_hook() {
    DETOUR_GUARD.call_once(|| {
        detour_attach().ok();
    });
}
