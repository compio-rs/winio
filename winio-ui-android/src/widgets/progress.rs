use inherit_methods_macro::inherit_methods;
use winio_handle::{AsContainer, impl_as_widget};
use winio_primitive::{Point, Size};

use crate::{
    BaseWidget, Result, current_activity, java::material::ProgressBar as AProgressBar, vm_exec,
};

#[derive(Debug)]
pub struct Progress {
    inner: BaseWidget<AProgressBar<'static>>,
}

#[inherit_methods(from = "self.inner")]
impl Progress {
    pub fn new(parent: impl AsContainer) -> Result<Self> {
        vm_exec(|env| {
            let act = current_activity(env)?;
            let widget = AProgressBar::new(env, act)?;
            let inner = BaseWidget::new_with_env(env, parent.as_container(), widget)?;
            inner.set_indeterminate(env, false)?;
            Ok(Self { inner })
        })
    }

    pub fn is_visible(&self) -> Result<bool>;

    pub fn set_visible(&mut self, visible: bool) -> Result<()>;

    pub fn is_enabled(&self) -> Result<bool>;

    pub fn set_enabled(&mut self, enabled: bool) -> Result<()>;

    pub fn loc(&self) -> Result<Point>;

    pub fn set_loc(&mut self, p: Point) -> Result<()>;

    pub fn size(&self) -> Result<Size>;

    pub fn set_size(&mut self, v: Size) -> Result<()>;

    pub fn preferred_size(&self) -> Result<Size>;

    pub fn tooltip(&self) -> Result<String>;

    pub fn set_tooltip(&mut self, s: impl AsRef<str>) -> Result<()>;

    pub fn minimum(&self) -> Result<usize> {
        vm_exec(|env| Ok(self.inner.get_min(env)? as _))
    }

    pub fn set_minimum(&mut self, minimum: usize) -> Result<()> {
        vm_exec(|env| {
            self.inner.set_min(env, minimum as i32)?;
            Ok(())
        })
    }

    pub fn maximum(&self) -> Result<usize> {
        vm_exec(|env| Ok(self.inner.get_max(env)? as _))
    }

    pub fn set_maximum(&mut self, maximum: usize) -> Result<()> {
        vm_exec(|env| {
            self.inner.set_max(env, maximum as i32)?;
            Ok(())
        })
    }

    pub fn pos(&self) -> Result<usize> {
        vm_exec(|env| Ok(self.inner.get_progress(env)? as _))
    }

    pub fn set_pos(&mut self, pos: usize) -> Result<()> {
        vm_exec(|env| {
            self.inner.set_progress(env, pos as i32)?;
            Ok(())
        })
    }

    pub fn is_indeterminate(&self) -> Result<bool> {
        vm_exec(|env| Ok(self.inner.is_indeterminate(env)?))
    }

    pub fn set_indeterminate(&mut self, indeterminate: bool) -> Result<()> {
        vm_exec(|env| {
            self.inner.set_indeterminate(env, indeterminate)?;
            Ok(())
        })
    }
}

impl_as_widget!(Progress, inner);
