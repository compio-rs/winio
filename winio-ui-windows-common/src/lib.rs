//! Windows common methods for winio.

#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg(windows)]

use windows_sys::Win32::Foundation::HWND;
use winio_handle::{AsRawWindow, AsWindow, RawWindow};

pub(crate) fn parent_handle(parent: Option<impl AsWindow>) -> Option<HWND> {
    parent.and_then(|parent| match parent.as_window().as_raw_window() {
        #[cfg(feature = "win32")]
        RawWindow::Win32(h) => Some(h),
        #[cfg(feature = "winui")]
        RawWindow::WinUI(window) => unsafe {
            use windows::core::Interface;
            use winui3::IWindowNative;
            Some(window.cast::<IWindowNative>().ok()?.WindowHandle().ok()?.0)
        },
        _ => unimplemented!(),
    })
}

mod accent;
pub use accent::*;

mod filebox;
pub use filebox::*;

mod msgbox;
pub use msgbox::*;

mod monitor;
pub use monitor::*;

mod canvas;
pub use canvas::*;

mod darkmode;
pub use darkmode::*;

mod resource;
pub use resource::*;

mod backdrop;
pub use backdrop::*;

mod runtime;
pub use runtime::*;

#[cfg(feature = "webview")]
mod webview;
#[cfg(feature = "webview")]
pub use webview::*;
