//! Hook GetMessage to wait for custom objects.

use std::sync::LazyLock;

use slim_detours_sys::SlimDetoursInlineHook;
use sync_unsafe_cell::SyncUnsafeCell;
use windows::core::Result;
use windows_sys::Win32::{
    Foundation::HWND,
    UI::WindowsAndMessaging::{GetMessageW, MSG},
};

type GetMessageWFn =
    unsafe extern "system" fn(msg: *mut MSG, hwnd: HWND, min: u32, max: u32) -> i32;

static TRUE_GET_MESSAGE_W: SyncUnsafeCell<GetMessageWFn> = SyncUnsafeCell::new(GetMessageW);

unsafe extern "system" fn get_message(msg: *mut MSG, hwnd: HWND, min: u32, max: u32) -> i32 {
    if let Some(res) = crate::runtime::run_runtime(msg, hwnd, min, max) {
        res
    } else {
        (*TRUE_GET_MESSAGE_W.get())(msg, hwnd, min, max)
    }
}

fn detour_attach() -> Result<()> {
    let res =
        unsafe { SlimDetoursInlineHook(1, TRUE_GET_MESSAGE_W.get().cast(), get_message as _) };
    windows::core::HRESULT(res).ok()
}

static DETOUR_GUARD: LazyLock<Result<()>> = LazyLock::new(detour_attach);

pub fn init_hook() -> bool {
    DETOUR_GUARD.is_ok()
}
