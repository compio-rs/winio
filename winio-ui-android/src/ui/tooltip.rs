use std::ops::{Deref, DerefMut};

#[derive(Debug)]
pub struct ToolTip<T> {
    inner: T,
}

impl<T> ToolTip<T> {
    pub fn tooltip(&self) -> String {
        todo!()
    }

    pub fn set_tooltip<S>(&mut self, _s: S)
    where
        S: AsRef<str>,
    {
        todo!()
    }

    pub fn new(_inner: T) -> Self {
        todo!()
    }
}

impl<T> Deref for ToolTip<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> DerefMut for ToolTip<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}