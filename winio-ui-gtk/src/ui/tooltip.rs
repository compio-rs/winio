use std::ops::{Deref, DerefMut};

use gtk4::prelude::WidgetExt;
use winio_handle::AsWidget;

pub struct ToolTip<T: AsWidget> {
    inner: T,
    widget: gtk4::Widget,
}

impl<T: AsWidget> ToolTip<T> {
    pub fn new(inner: T) -> Self {
        let widget = inner.as_widget().to_gtk();
        Self { inner, widget }
    }

    pub fn tooltip(&self) -> String {
        self.widget
            .tooltip_text()
            .map(|s| s.to_string())
            .unwrap_or_default()
    }

    pub fn set_tooltip(&mut self, s: impl AsRef<str>) {
        let s = s.as_ref();
        self.widget
            .set_tooltip_text(if s.is_empty() { None } else { Some(s) });
    }
}

impl<T: AsWidget> Drop for ToolTip<T> {
    fn drop(&mut self) {
        self.widget.set_tooltip_text(None);
    }
}

impl<T: AsWidget> Deref for ToolTip<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T: AsWidget> DerefMut for ToolTip<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
