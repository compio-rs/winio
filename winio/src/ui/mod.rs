cfg_if::cfg_if! {
    if #[cfg(windows)] {
        use winio_ui_win32 as sys;
    } else if #[cfg(target_os = "macos")] {
        use winio_ui_app_kit as sys;
    } else {
        #[cfg(any(
            all(not(feature = "gtk"), not(feature = "qt")),
            all(feature = "gtk", feature = "qt")
        ))]
        compile_error!("You must choose only one of these features: [\"gtk\", \"qt\"]");

        cfg_if::cfg_if! {
            if #[cfg(feature = "qt")] {
                use winio_ui_qt as sys;
            } else {
                use winio_ui_gtk as sys;
            }
        }
    }
}

pub use sys::*;

mod canvas;
mod ext;
mod filebox;
mod msgbox;

pub mod export {
    pub use super::{
        canvas::*,
        ext::*,
        filebox::*,
        msgbox::*,
        sys::{Brush, Pen},
    };
}
