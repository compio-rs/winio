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

use crate::ColorTheme;

pub fn color_theme() -> ColorTheme {
    if is_dark() {
        ColorTheme::Dark
    } else {
        ColorTheme::Light
    }
}

pub type RawWindow = *mut QWidget;
