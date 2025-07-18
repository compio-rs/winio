use std::ops::{Deref, DerefMut};

use gtk4::prelude::WidgetExt;
use winio_handle::AsWidget;

pub struct ToolTip<T: AsWidget> {
    inner: T,
}

impl<T: AsWidget> ToolTip<T> {
    pub fn new(inner: T) -> Self {
        Self { inner }
    }

    pub fn tooltip(&self) -> String {
        let widget = self.inner.as_widget().to_gtk();
        widget
            .tooltip_text()
            .map(|s| s.to_string())
            .unwrap_or_default()
    }

    pub fn set_tooltip(&mut self, s: impl AsRef<str>) {
        let widget = self.inner.as_widget().to_gtk();
        let s = s.as_ref();
        widget.set_tooltip_text(if s.is_empty() { None } else { Some(s) });
    }
}

impl<T: AsWidget> Drop for ToolTip<T> {
    fn drop(&mut self) {
        let widget = self.inner.as_widget().to_gtk();
        widget.set_tooltip_text(None);
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
