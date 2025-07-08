//! A single-threaded asynchronous GUI runtime, based on [`compio`] and ELM
//! design.

#![cfg_attr(docsrs, feature(doc_cfg, doc_auto_cfg))]
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
        #[cfg(any(
            all(not(feature = "win32"), not(feature = "winui")),
            all(feature = "win32", feature = "winui")
        ))]
        compile_error!("You must choose only one of these features: [\"win32\", \"winui\"]");

        cfg_if::cfg_if! {
            if #[cfg(feature = "winui")] {
                use winio_ui_winui as sys;
            } else {
                use winio_ui_win32 as sys;
            }
        }
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

pub mod ui;

pub mod widgets;

/// For blanket imports.
pub mod prelude {
    pub use crate::{elm::*, handle::*, layout::*, primitive::*, ui::*, widgets::*};
}
