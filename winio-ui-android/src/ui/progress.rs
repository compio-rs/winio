use inherit_methods_macro::inherit_methods;
use winio_handle::{AsContainer, impl_as_widget};
use winio_primitive::{Point, Size};

use crate::{BaseWidget, Result, vm_exec};

#[derive(Debug)]
pub struct Progress {
    inner: BaseWidget,
}

#[inherit_methods(from = "self.inner")]
impl Progress {
    const WIDGET_CLASS: &'static str = "android/widget/ProgressBar";

    pub fn new(parent: impl AsContainer) -> Result<Self> {
        vm_exec(|env| {
            let inner = BaseWidget::new_with_env(env, parent.as_container(), Self::WIDGET_CLASS)?;
            env.call_method(
                inner.as_obj(),
                jni::jni_str!("setIndeterminate"),
                jni::jni_sig!("(Z)V"),
                &[false.into()],
            )?
            .v()?;
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
        vm_exec(|env| {
            Ok(env
                .call_method(
                    self.inner.as_obj(),
                    jni::jni_str!("getMin"),
                    jni::jni_sig!("()I"),
                    &[],
                )?
                .i()? as _)
        })
    }

    pub fn set_minimum(&mut self, minimum: usize) -> Result<()> {
        vm_exec(|env| {
            env.call_method(
                self.inner.as_obj(),
                jni::jni_str!("setMin"),
                jni::jni_sig!("(I)V"),
                &[(minimum as i32).into()],
            )?
            .v()?;
            Ok(())
        })
    }

    pub fn maximum(&self) -> Result<usize> {
        vm_exec(|env| {
            Ok(env
                .call_method(
                    self.inner.as_obj(),
                    jni::jni_str!("getMax"),
                    jni::jni_sig!("()I"),
                    &[],
                )?
                .i()? as _)
        })
    }

    pub fn set_maximum(&mut self, maximum: usize) -> Result<()> {
        vm_exec(|env| {
            env.call_method(
                self.inner.as_obj(),
                jni::jni_str!("setMax"),
                jni::jni_sig!("(I)V"),
                &[(maximum as i32).into()],
            )?
            .v()?;
            Ok(())
        })
    }

    pub fn pos(&self) -> Result<usize> {
        vm_exec(|env| {
            Ok(env
                .call_method(
                    self.inner.as_obj(),
                    jni::jni_str!("getProgress"),
                    jni::jni_sig!("()I"),
                    &[],
                )?
                .i()? as _)
        })
    }

    pub fn set_pos(&mut self, pos: usize) -> Result<()> {
        vm_exec(|env| {
            env.call_method(
                self.inner.as_obj(),
                jni::jni_str!("setProgress"),
                jni::jni_sig!("(I)V"),
                &[(pos as i32).into()],
            )?
            .v()?;
            Ok(())
        })
    }

    pub fn is_indeterminate(&self) -> Result<bool> {
        vm_exec(|env| {
            Ok(env
                .call_method(
                    self.inner.as_obj(),
                    jni::jni_str!("isIndeterminate"),
                    jni::jni_sig!("()Z"),
                    &[],
                )?
                .z()?)
        })
    }

    pub fn set_indeterminate(&mut self, indeterminate: bool) -> Result<()> {
        vm_exec(|env| {
            env.call_method(
                self.inner.as_obj(),
                jni::jni_str!("setIndeterminate"),
                jni::jni_sig!("(Z)V"),
                &[indeterminate.into()],
            )?
            .v()?;
            Ok(())
        })
    }
}

impl_as_widget!(Progress, inner);
