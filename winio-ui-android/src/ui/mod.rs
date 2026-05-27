//! Android UI widgets for winio.

use winio_primitive::ColorTheme;

pub fn color_theme() -> crate::Result<ColorTheme> {
    Ok(ColorTheme::Light)
}

mod accent;
pub use accent::*;

mod button;
pub use button::*;

mod canvas;
pub use canvas::*;

mod combo_box;
pub use combo_box::*;

mod edit;
pub use edit::*;

mod filebox;
pub use filebox::*;

mod label;
pub use label::*;

mod link_label;
pub use link_label::*;

mod list_box;
pub use list_box::*;

mod monitor;
pub use monitor::*;

mod msgbox;
pub use msgbox::*;

mod progress;
pub use progress::*;

mod scroll_bar;
pub use scroll_bar::*;

mod scroll_view;
pub use scroll_view::*;

mod slider;
pub use slider::*;

mod tab_view;
pub use tab_view::*;

mod view;
pub use view::*;

mod widget;
pub(crate) use widget::*;

mod window;
pub use window::*;
