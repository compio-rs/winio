use std::ops::{Deref, DerefMut};

use objc2::rc::Retained;
use objc2_foundation::NSString;
use winio_handle::{AsRawWidget, AsWidget};

use crate::from_nsstring;

pub struct ToolTip<T: AsWidget> {
    inner: T,
    text: Option<Retained<NSString>>,
}

impl<T: AsWidget> ToolTip<T> {
    pub fn new(inner: T) -> Self {
        Self { inner, text: None }
    }

    pub fn tooltip(&self) -> String {
        self.text.as_deref().map(from_nsstring).unwrap_or_default()
    }

    pub fn set_tooltip(&mut self, s: impl AsRef<str>) {
        let s = s.as_ref();
        self.text = if s.is_empty() {
            None
        } else {
            Some(NSString::from_str(s))
        };
        for handle in self.inner.iter_widgets() {
            let view = handle.as_raw_widget();
            unsafe {
                view.setToolTip(self.text.as_deref());
            }
        }
    }
}

impl<T: AsWidget> Drop for ToolTip<T> {
    fn drop(&mut self) {
        for handle in self.inner.iter_widgets() {
            let view = handle.as_raw_widget();
            unsafe {
                view.setToolTip(None);
            }
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
