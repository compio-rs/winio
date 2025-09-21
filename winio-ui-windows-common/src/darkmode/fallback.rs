#![allow(clippy::missing_safety_doc)]

use windows_sys::{
    Win32::{
        Foundation::{HWND, LPARAM, S_OK},
        UI::Controls::PFTASKDIALOGCALLBACK,
    },
    core::HRESULT,
};

use super::PreferredAppMode;

pub fn is_dark_mode_allowed_for_app() -> bool {
    false
}

pub(crate) const TASK_DIALOG_CALLBACK: PFTASKDIALOGCALLBACK = None;

pub unsafe fn control_use_dark_mode(_: HWND, _: bool) {}

pub fn set_preferred_app_mode(_: PreferredAppMode) -> PreferredAppMode {
    PreferredAppMode::Default
}

pub unsafe fn window_use_dark_mode(_: HWND) -> HRESULT {
    S_OK
}

pub unsafe fn children_refresh_dark_mode(_: HWND, _: LPARAM) {}

pub fn init_dark() {}
