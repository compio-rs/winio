//! Android UI widgets for winio.

mod accent;
mod button;
mod canvas;
mod check_box;
mod combo_box;
mod edit;
mod filebox;
mod label;
mod list_box;
mod monitor;
mod msgbox;
mod progress;
mod radio_button;
mod scroll_bar;
mod slider;
mod text_box;
mod tooltip;
mod widget;
mod window;

pub use accent::*;
pub use button::*;
pub use canvas::*;
pub use check_box::*;
pub use combo_box::*;
pub use edit::*;
pub use filebox::*;
pub use label::*;
pub use list_box::*;
pub use monitor::*;
pub use msgbox::*;
pub use progress::*;
pub use radio_button::*;
pub use scroll_bar::*;
pub use slider::*;
pub use text_box::*;
pub use tooltip::*;
pub(crate) use widget::*;
pub use window::*;
use winio_primitive::ColorTheme;

pub fn color_theme() -> ColorTheme {
    ColorTheme::Light
}
