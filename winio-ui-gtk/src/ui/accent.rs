use winio_primitive::Color;

use crate::{Error, Result};

/// Get the accent color.
pub fn accent_color() -> Result<Color> {
    Err(Error::NoColorTheme)
}
