//! A single-threaded asynchronous GUI runtime, based on [`compio`] and ELM
//! design.

#![cfg_attr(docsrs, feature(doc_cfg))]
#![warn(missing_docs)]

#[doc(no_inline)]
pub use compio;
#[doc(inline)]
pub use winio_elm as elm;
#[doc(inline)]
pub use winio_handle as handle;
#[doc(inline)]
pub use winio_layout as layout;
#[doc(inline)]
pub use winio_primitive as primitive;

cfg_if::cfg_if! {
    if #[cfg(windows)] {
        #[cfg(all(feature = "win32", feature = "winui"))]
        compile_error!("You must choose only one of these features: [\"win32\", \"winui\"]");

        cfg_if::cfg_if! {
            if #[cfg(feature = "winui")] {
                use winio_ui_winui as sys;
            } else if #[cfg(feature = "win32")] {
                use winio_ui_win32 as sys;
            } else {
                mod stub;
                use stub as sys;
            }
        }
    } else if #[cfg(target_os = "macos")] {
        use winio_ui_app_kit as sys;
    } else {
        #[cfg(all(feature = "gtk", feature = "qt"))]
        compile_error!("You must choose only one of these features: [\"gtk\", \"qt\"]");

        cfg_if::cfg_if! {
            if #[cfg(feature = "qt")] {
                use winio_ui_qt as sys;
            } else if #[cfg(feature = "gtk")] {
                use winio_ui_gtk as sys;
            } else {
                mod stub;
                use stub as sys;
            }
        }
    }
}

pub mod ui;

pub mod widgets;

/// For blanket imports.
pub mod prelude {
    pub use crate::{Error, Result, elm::*, handle::*, layout::*, primitive::*, ui::*, widgets::*};
}

pub use sys::{Error, Result};
