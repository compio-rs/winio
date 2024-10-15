mod window;
pub use window::*;

mod canvas;
pub use canvas::*;

mod widget;
pub use widget::*;

mod msgbox;
pub use msgbox::*;

mod filebox;
pub use filebox::*;

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

use crate::ColorTheme;

pub fn color_theme() -> ColorTheme {
    if is_dark() {
        ColorTheme::Dark
    } else {
        ColorTheme::Light
    }
}

/// Pointer to `QWidget`.
pub type RawWindow = *mut QWidget;
