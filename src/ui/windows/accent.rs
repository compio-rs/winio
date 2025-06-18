use windows::{
    UI::ViewManagement::{UIColorType, UISettings},
    core::Result,
};

use crate::Color;

/// Get the accent color.
pub fn accent_color() -> Option<Color> {
    accent_impl().ok()
}

fn accent_impl() -> Result<Color> {
    let settings = UISettings::new()?;
    let accent = settings.GetColorValue(UIColorType::Accent)?;
    Ok(Color::new(accent.R, accent.G, accent.B, accent.A))
}
