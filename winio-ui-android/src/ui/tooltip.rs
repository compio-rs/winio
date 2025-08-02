use {
    super::{super::JObjectExt, vm_exec_on_ui_thread},
    std::ops::{Deref, DerefMut},
    winio_handle::{AsWidget, RawWidget},
};

#[derive(Debug)]
pub struct ToolTip<T> {
    inner: T,
    tooltip: RawWidget,
}

//noinspection SpellCheckingInspection
impl<T> ToolTip<T> {
    const WIDGET_CLASS: &'static str = "rs/compio/winio/Tooltip";

    pub fn tooltip(&self) -> String {
        let w = self.tooltip.clone();
        vm_exec_on_ui_thread(move |mut env, _| {
            env.call_method(w.as_obj(), "getTooltip", "()Ljava/lang/CharSequence;", &[])?
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
        let w = self.tooltip.clone();
        vm_exec_on_ui_thread(move |mut env, _| {
            let text = env.new_string(&text)?;
            env.call_method(
                w.as_obj(),
                "setTooltip",
                "(Ljava/lang/CharSequence;)V",
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
        let w = (&*inner.as_widget()).clone();
        let tooltip = vm_exec_on_ui_thread(move |mut env, _| {
            let tooltip = env.new_object(
                Self::WIDGET_CLASS,
                "(Landroid/view/View;)V",
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
