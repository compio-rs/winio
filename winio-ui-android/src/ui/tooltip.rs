use std::ops::{Deref, DerefMut};

use winio_handle::AsWidget;

use super::{super::JObjectExt, vm_exec, vm_exec_on_ui_thread};
use crate::GlobalRef;

#[derive(Debug)]
pub struct ToolTip<T> {
    inner: T,
    tooltip: GlobalRef,
}

// noinspection SpellCheckingInspection
impl<T> ToolTip<T> {
    const WIDGET_CLASS: &'static str = "rs/compio/winio/Tooltip";

    fn duplicate_tooltip(&self) -> GlobalRef {
        vm_exec(|env, _| env.new_global_ref(self.tooltip.as_obj())).unwrap()
    }

    pub fn tooltip(&self) -> String {
        let w = self.duplicate_tooltip();
        vm_exec_on_ui_thread(move |mut env, _| {
            env.call_method(
                w.as_obj(),
                jni::jni_str!("getTooltip"),
                jni::jni_sig!("()Ljava/lang/CharSequence;"),
                &[],
            )?
            .l()?
            .to(&mut env)
        })
        .unwrap()
    }

    pub fn set_tooltip<S>(&self, text: S)
    where
        S: AsRef<str>,
    {
        let text = text.as_ref().to_owned();
        let w = self.duplicate_tooltip();
        vm_exec_on_ui_thread(move |env, _| {
            let text = env.new_string(&text)?;
            env.call_method(
                w.as_obj(),
                jni::jni_str!("setTooltip"),
                jni::jni_sig!("(Ljava/lang/CharSequence;)V"),
                &[(&text).into()],
            )?
            .v()
        })
        .unwrap();
    }

    pub fn new(inner: T) -> Self
    where
        T: AsWidget,
    {
        let w =
            vm_exec(|env, _| env.new_global_ref(inner.as_widget().to_android().as_obj())).unwrap();
        let tooltip = vm_exec_on_ui_thread(move |env, _| {
            let tooltip = env.new_object(
                jni::strings::JNIString::from(Self::WIDGET_CLASS),
                jni::jni_sig!("(Landroid/view/View;)V"),
                &[w.as_obj().into()],
            )?;
            env.new_global_ref(tooltip)
        })
        .unwrap();

        Self { inner, tooltip }
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
