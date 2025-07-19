use std::{
    ops::{Deref, DerefMut},
    pin::Pin,
};

use winio_handle::AsWidget;

use crate::{QString, QWidget};

#[derive(Debug)]
pub struct ToolTip<T: AsWidget> {
    inner: T,
    text: QString,
}

impl<T: AsWidget> ToolTip<T> {
    pub fn new(inner: T) -> Self {
        Self {
            inner,
            text: QString::from(String::new()),
        }
    }

    pub fn tooltip(&self) -> String {
        (&self.text).into()
    }

    pub fn set_tooltip(&mut self, s: impl AsRef<str>) {
        self.text = s.as_ref().into();
        for w in self.inner.iter_widgets() {
            let w = unsafe { Pin::new_unchecked(&mut *w.as_qt::<QWidget>()) };
            w.setToolTip(&self.text);
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
