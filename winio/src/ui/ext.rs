use winio_primitive::{Color, ColorTheme, Monitor};

/// Extension trait for [`Monitor`].
pub trait MonitorExt: Sized {
    /// Retrieve all monitors information.
    fn all() -> Vec<Self>;
}

impl MonitorExt for Monitor {
    fn all() -> Vec<Self> {
        crate::sys::monitor_get_all()
    }
}

/// Extension trait for [`ColorTheme`].
pub trait ColorThemeExt {
    /// Get current color theme.
    fn current() -> Self;
}

impl ColorThemeExt for ColorTheme {
    fn current() -> Self {
        crate::sys::color_theme()
    }
}

/// Extension trait for [`Color`].
pub trait ColorExt: Sized {
    /// Get accent color.
    fn accent() -> Option<Self>;
}

impl ColorExt for Color {
    fn accent() -> Option<Self> {
        crate::sys::accent_color()
    }
}
