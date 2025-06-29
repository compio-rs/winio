use crate::{ColorTheme, Monitor};

/// Extension trait for [`Monitor`].
pub trait MonitorExt: Sized {
    /// Retrieve all monitors information.
    fn all() -> Vec<Self>;
}

impl MonitorExt for Monitor {
    fn all() -> Vec<Self> {
        crate::ui::monitor_get_all()
    }
}

/// Extension trait for [`ColorTheme`].
pub trait ColorThemeExt {
    /// Get current color theme.
    fn current() -> Self;
}

impl ColorThemeExt for ColorTheme {
    fn current() -> Self {
        crate::ui::color_theme()
    }
}
