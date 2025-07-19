use std::ops::{Deref, DerefMut};

use gtk4::prelude::WidgetExt;
use winio_handle::AsWidget;

pub struct ToolTip<T: AsWidget> {
    inner: T,
    text: String,
}

impl<T: AsWidget> ToolTip<T> {
    pub fn new(inner: T) -> Self {
        Self {
            inner,
            text: String::new(),
        }
    }

    pub fn tooltip(&self) -> String {
        self.text.clone()
    }

    pub fn set_tooltip(&mut self, s: impl AsRef<str>) {
        let s = s.as_ref();
        self.text = s.to_string();
        let s = if s.is_empty() { None } else { Some(s) };
        for w in self.inner.iter_widgets() {
            let widget = w.to_gtk();
            widget.set_tooltip_text(s);
        }
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
