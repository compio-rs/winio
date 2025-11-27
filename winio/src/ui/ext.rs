use winio_primitive::{Color, ColorTheme, Monitor};

use crate::sys::Result;

/// Extension trait for [`Monitor`].
pub trait MonitorExt: Sized {
    /// Retrieve all monitors information.
    fn all() -> Result<Vec<Self>>;
}

impl MonitorExt for Monitor {
    fn all() -> Result<Vec<Self>> {
        crate::sys::monitor_get_all()
    }
}

/// Extension trait for [`ColorTheme`].
pub trait ColorThemeExt: Sized {
    /// Get current color theme.
    fn current() -> Result<Self>;
}

impl ColorThemeExt for ColorTheme {
    fn current() -> Result<Self> {
        crate::sys::color_theme()
    }
}

/// Extension trait for [`Color`].
pub trait ColorExt: Sized {
    /// Get accent color.
    fn accent() -> Result<Self>;
}

impl ColorExt for Color {
    fn accent() -> Result<Self> {
        crate::sys::accent_color()
    }
}
