use windows::UI::ViewManagement::{UIColorType, UISettings};
use winio_primitive::Color;

/// Get the accent color.
pub fn accent_color() -> crate::Result<Color> {
    let settings = UISettings::new()?;
    let accent = settings.GetColorValue(UIColorType::Accent)?;
    Ok(Color::new(accent.R, accent.G, accent.B, accent.A))
}
