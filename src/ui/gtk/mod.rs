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

/// GTK [`Window`].
///
/// [`Window`]: gtk4::Window
pub type RawWindow = gtk4::Window;

use std::cell::Cell;

use crate::ColorTheme;

thread_local! {
    pub(crate) static COLOR_THEME: Cell<ColorTheme> = const { Cell::new(ColorTheme::Light) };
}

pub fn color_theme() -> ColorTheme {
    COLOR_THEME.get()
}
