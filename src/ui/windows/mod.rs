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

pub type RawWindow = windows_sys::Win32::Foundation::HWND;
