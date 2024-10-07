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

mod drawing;

mod msgbox;

mod canvas;

pub mod export {
    pub use super::{
        canvas::*,
        drawing::*,
        msgbox::*,
        sys::{Brush, CustomButton, DrawingContext, FileBox, FileFilter, MessageBox, Pen},
    };
}

#[cfg(not(windows))]
mod callback;
#[cfg(not(windows))]
pub(crate) use callback::*;
