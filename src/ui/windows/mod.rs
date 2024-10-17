mod canvas;
pub use canvas::*;

pub(crate) mod darkmode;
pub(crate) mod dpi;
pub(crate) mod font;

mod msgbox;
pub use msgbox::*;

mod filebox;
pub use filebox::*;

mod window;
pub use window::*;

mod button;
pub use button::*;

mod edit;
pub use edit::*;

mod label;
pub use label::*;

mod progress;
pub use progress::*;

mod combo_box;
pub use combo_box::*;

mod check_box;
pub use check_box::*;

use crate::ColorTheme;

pub fn color_theme() -> ColorTheme {
    unsafe {
        if darkmode::is_dark_mode_allowed_for_app() {
            ColorTheme::Dark
        } else {
            ColorTheme::Light
        }
    }
}

/// Raw window handle.
pub type RawWindow = windows_sys::Win32::Foundation::HWND;
