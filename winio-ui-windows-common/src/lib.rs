//! Windows common methods for winio.

#![cfg_attr(docsrs, feature(doc_cfg, doc_auto_cfg))]
#![cfg(windows)]

use windows_sys::Win32::Foundation::HWND;
use winio_handle::{AsRawWindow, AsWindow, RawWindow};

pub(crate) fn parent_handle(parent: Option<impl AsWindow>) -> Option<HWND> {
    parent.and_then(|parent| match parent.as_window().as_raw_window() {
        #[cfg(feature = "win32")]
        RawWindow::Win32(h) => Some(h),
        #[cfg(feature = "winui")]
        RawWindow::WinUI(window) => Some(window.AppWindow().ok()?.Id().ok()?.Value as _),
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
