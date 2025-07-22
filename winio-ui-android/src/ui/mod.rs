//! Android UI widgets for winio.

mod accent;
mod activity;
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
mod text_box;
mod tooltip;
mod widget;
mod window;

use winio_primitive::ColorTheme;
pub use {
    accent::*, activity::*, button::*, canvas::*, check_box::*, combo_box::*, edit::*, filebox::*,
    label::*, list_box::*, monitor::*, msgbox::*, progress::*, radio_button::*, scroll_bar::*,
    text_box::*, tooltip::*, widget::*, window::*,
};

pub fn color_theme() -> ColorTheme {
    ColorTheme::Dark
}
