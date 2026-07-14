use objc2_ui_kit::UIColor;
use winio_primitive::Color;

use crate::{Result, catch};

/// Get the accent color.
pub fn accent_color() -> Result<Color> {
    catch(|| {
        let accent = UIColor::tintColor();
        let mut r: f64 = 0.0;
        let mut g: f64 = 0.0;
        let mut b: f64 = 0.0;
        let mut a: f64 = 0.0;
        unsafe {
            accent.getRed_green_blue_alpha(&mut r, &mut g, &mut b, &mut a);
        }
        Ok(Color::new(
            (r * 255.0) as u8,
            (g * 255.0) as u8,
            (b * 255.0) as u8,
            (a * 255.0) as u8,
        ))
    })
    .flatten()
}
