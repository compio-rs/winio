use windows_sys::{
    Win32::{
        Foundation::{HWND, LPARAM, S_OK},
        UI::Controls::PFTASKDIALOGCALLBACK,
    },
    core::HRESULT,
};

use super::PreferredAppMode;

pub unsafe fn is_dark_mode_allowed_for_app() -> bool {
    false
}

pub const TASK_DIALOG_CALLBACK: PFTASKDIALOGCALLBACK = None;

pub unsafe fn control_use_dark_mode(_: HWND, _: bool) {}

pub unsafe fn set_preferred_app_mode(_: PreferredAppMode) -> PreferredAppMode {
    PreferredAppMode::Default
}

pub unsafe fn window_use_dark_mode(_: HWND) -> HRESULT {
    S_OK
}

pub unsafe fn children_refresh_dark_mode(_: HWND, _: LPARAM) {}

pub unsafe fn init_dark() {}
