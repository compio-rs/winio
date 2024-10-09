cfg_if::cfg_if! {
    if #[cfg(windows)] {
        #[path = "windows/mod.rs"]
        mod sys;
    } else if #[cfg(target_os = "macos")] {
        #[path = "mac/mod.rs"]
        mod sys;
    } else {
        #[cfg(all(not(feature = "gtk"), not(feature = "qt")))]
        compile_error!("You must choose one of these features: [\"gtk\", \"qt\"]");

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
mod drawing;
mod filebox;
mod msgbox;
mod window_handle;

pub mod export {
    pub use super::{
        canvas::*,
        drawing::*,
        filebox::*,
        msgbox::*,
        sys::{Brush, Pen, RawWindow},
        window_handle::*,
    };
}

#[cfg(not(windows))]
mod callback;
#[cfg(not(windows))]
pub(crate) use callback::*;
