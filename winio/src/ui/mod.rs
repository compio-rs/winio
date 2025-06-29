cfg_if::cfg_if! {
    if #[cfg(windows)] {
        #[path = "windows/mod.rs"]
        mod sys;
    } else if #[cfg(target_os = "macos")] {
        #[path = "mac/mod.rs"]
        mod sys;
    } else {
        #[cfg(any(
            all(not(feature = "gtk"), not(feature = "qt")),
            all(feature = "gtk", feature = "qt")
        ))]
        compile_error!("You must choose only one of these features: [\"gtk\", \"qt\"]");

        cfg_if::cfg_if! {
            if #[cfg(feature = "qt")] {
                #[path = "qt/mod.rs"]
                mod sys;
            } else {
                #[path = "gtk/mod.rs"]
                mod sys;
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
        sys::{Brush, Pen, accent_color},
    };
}

#[cfg(not(windows))]
mod callback;
#[cfg(not(windows))]
pub(crate) use callback::*;
