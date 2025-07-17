use std::{
    ops::{Deref, DerefMut},
    pin::Pin,
};

use winio_handle::AsWidget;

use crate::QWidget;

pub struct ToolTip<T: AsWidget> {
    inner: T,
}

impl<T: AsWidget> ToolTip<T> {
    pub fn new(inner: T) -> Self {
        Self { inner }
    }

    fn as_ref_qwidget(&self) -> &QWidget {
        unsafe { &*self.inner.as_widget().as_qt::<QWidget>() }
    }

    fn pin_mut_qwidget(&mut self) -> Pin<&mut QWidget> {
        unsafe { Pin::new_unchecked(&mut *self.inner.as_widget().as_qt::<QWidget>()) }
    }

    pub fn tooltip(&self) -> String {
        self.as_ref_qwidget().toolTip().into()
    }

    pub fn set_tooltip(&mut self, s: impl AsRef<str>) {
        self.pin_mut_qwidget().setToolTip(&s.as_ref().into());
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
