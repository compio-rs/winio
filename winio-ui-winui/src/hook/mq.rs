//! Hook GetMessage to wait for custom objects.

use slim_detours_sys::SlimDetoursInlineHook;
use sync_unsafe_cell::SyncUnsafeCell;
use windows::core::{Error, Result};
use windows_sys::Win32::{
    Foundation::HWND,
    UI::WindowsAndMessaging::{GetMessageW, MSG},
};

type GetMessageWFn =
    unsafe extern "system" fn(msg: *mut MSG, hwnd: HWND, min: u32, max: u32) -> i32;

static TRUE_GET_MESSAGE_W: SyncUnsafeCell<GetMessageWFn> = SyncUnsafeCell::new(GetMessageW);

unsafe extern "system" fn get_message(msg: *mut MSG, hwnd: HWND, min: u32, max: u32) -> i32 {
    crate::runtime::run_runtime(msg, hwnd, min, max)
}

pub struct HookGuard;

impl HookGuard {
    pub fn new() -> Result<Self> {
        attach_hook()?;
        Ok(Self)
    }
}

impl Drop for HookGuard {
    fn drop(&mut self) {
        detach_hook().unwrap();
    }
}

fn attach_hook() -> Result<()> {
    let res =
        unsafe { SlimDetoursInlineHook(1, TRUE_GET_MESSAGE_W.get().cast(), get_message as _) };
    if res < 0 {
        Err(Error::from_hresult(windows::core::HRESULT(res)))
    } else {
        Ok(())
    }
}

fn detach_hook() -> Result<()> {
    let res =
        unsafe { SlimDetoursInlineHook(0, TRUE_GET_MESSAGE_W.get().cast(), get_message as _) };
    if res < 0 {
        Err(Error::from_hresult(windows::core::HRESULT(res)))
    } else {
        Ok(())
    }
}
