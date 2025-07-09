use std::ptr::null_mut;

use windows_sys::Win32::{
    Foundation::HMODULE,
    System::LibraryLoader::{
        GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS, GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT,
        GetModuleHandleExW,
    },
};

/// Get the handle of the current executable or DLL.
pub fn get_current_module_handle() -> HMODULE {
    let mut module: HMODULE = null_mut();
    unsafe {
        GetModuleHandleExW(
            GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS | GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT,
            get_current_module_handle as *const _,
            &mut module,
        )
    };
    module
}
