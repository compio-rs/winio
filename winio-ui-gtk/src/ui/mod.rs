use std::cell::Cell;

use winio_primitive::ColorTheme;
thread_local! {
    pub(crate) static COLOR_THEME: Cell<ColorTheme> = const { Cell::new(ColorTheme::Light) };
}

pub fn color_theme() -> ColorTheme {
    COLOR_THEME.get()
}

mod window;
pub use window::*;

mod canvas;
pub use canvas::*;

mod widget;
pub(crate) use widget::*;

mod monitor;
pub use monitor::*;

mod msgbox;
pub use msgbox::*;

mod filebox;
pub use filebox::*;

mod button;
pub use button::*;

mod edit;
pub use edit::*;

mod text_box;
pub use text_box::*;

mod label;
pub use label::*;

mod progress;
pub use progress::*;

mod combo_box;
pub use combo_box::*;

mod list_box;
pub use list_box::*;

mod check_box;
pub use check_box::*;

mod accent;
pub use accent::*;
