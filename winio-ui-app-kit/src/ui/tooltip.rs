use std::ops::{Deref, DerefMut};

use objc2_foundation::NSString;
use winio_handle::{AsRawWidget, AsWidget};

use crate::from_nsstring;

pub struct ToolTip<T: AsWidget> {
    inner: T,
}

impl<T: AsWidget> ToolTip<T> {
    pub fn new(inner: T) -> Self {
        Self { inner }
    }

    pub fn tooltip(&self) -> String {
        let view = self.inner.as_widget().as_raw_widget();
        unsafe {
            view.toolTip()
                .map(|s| from_nsstring(&s))
                .unwrap_or_default()
        }
    }

    pub fn set_tooltip(&mut self, s: impl AsRef<str>) {
        let view = self.inner.as_widget().as_raw_widget();
        let s = s.as_ref();
        unsafe {
            if s.is_empty() {
                view.setToolTip(None);
            } else {
                view.setToolTip(Some(&NSString::from_str(s)));
            }
        }
    }
}

impl<T: AsWidget> Drop for ToolTip<T> {
    fn drop(&mut self) {
        let view = self.inner.as_widget().as_raw_widget();
        unsafe {
            view.setToolTip(None);
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
