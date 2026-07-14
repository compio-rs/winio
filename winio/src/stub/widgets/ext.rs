use winio_primitive::{Color, ColorTheme, Monitor};

use crate::stub::{Result, not_impl};

pub fn monitor_get_all() -> Result<Vec<Monitor>> {
    not_impl()
}

pub fn color_theme() -> Result<ColorTheme> {
    not_impl()
}

pub fn accent_color() -> Result<Color> {
    not_impl()
}
