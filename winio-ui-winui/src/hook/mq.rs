//! Hook GetMessage to wait for custom objects.

use std::sync::LazyLock;

use compio_log::error;
use slim_detours_sys::SlimDetoursInlineHook;
use sync_unsafe_cell::SyncUnsafeCell;
use windows::{
    Win32::{
        Foundation::HANDLE,
        Storage::Packaging::Appx::{
            AppPolicyGetWindowingModel, AppPolicyWindowingModel_ClassicDesktop,
            AppPolicyWindowingModel_None,
        },
    },
    core::Result,
};
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
    let desktop = is_desktop().unwrap_or_default();
    desktop && {
        let res = &*DETOUR_GUARD;
        if let Err(_e) = res {
            error!("Failed to hook GetMessageW: {_e:?}");
        }
        res.is_ok()
    }
}

fn is_desktop() -> Result<bool> {
    let mut policy = AppPolicyWindowingModel_None;
    unsafe { AppPolicyGetWindowingModel(GetCurrentThreadEffectiveToken(), &mut policy) }.ok()?;
    Ok(policy == AppPolicyWindowingModel_ClassicDesktop)
}

#[inline]
#[allow(non_snake_case)]
fn GetCurrentThreadEffectiveToken() -> HANDLE {
    HANDLE(-6_isize as _)
}
