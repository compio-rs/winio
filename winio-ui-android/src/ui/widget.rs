use {
    super::{super::JObjectExt, vm_exec, vm_exec_on_ui_thread},
    std::ops::Deref,
    winio_handle::RawWidget,
    winio_primitive::Size,
};

#[derive(Clone, Debug)]
pub struct BaseWidget {
    inner: RawWidget,
}

impl BaseWidget {
    pub(crate) fn new(inner: RawWidget) -> Self {
        Self { inner }
    }

    pub(crate) fn hash_code(&self) -> i32 {
        vm_exec(|mut env, _| {
            env.call_method(self.inner.as_obj(), "hashCode", "()I", &[])?
                .i()
        })
        .unwrap()
    }

    pub(crate) fn size(&self) -> Size {
        let w = self.inner.clone();
        vm_exec_on_ui_thread(move |mut env, _| {
            env.call_method(w.as_obj(), "getSize", "()[D", &[])?
                .l()?
                .to(&mut env)
        })
        .unwrap()
    }

    pub(crate) fn set_size(&mut self, size: Size) {
        let w = self.inner.clone();
        vm_exec_on_ui_thread(move |mut env, _| {
            env.call_method(
                w.as_obj(),
                "setSize",
                "(DD)V",
                &[size.width.into(), size.height.into()],
            )?
            .v()
        })
        .unwrap();
    }

    pub(crate) fn is_visible(&self) -> bool {
        let w = self.inner.clone();
        vm_exec_on_ui_thread(move |mut env, _| {
            env.call_method(w.as_obj(), "isVisible", "()[D", &[])?.z()
        })
        .unwrap()
    }

    pub(crate) fn set_visible(&mut self, visible: bool) {
        let w = self.inner.clone();
        vm_exec_on_ui_thread(move |mut env, _| {
            env.call_method(w.as_obj(), "setVisible", "(Z)V", &[visible.into()])?
                .v()
        })
        .unwrap();
    }

    pub(crate) fn text(&self) -> String {
        let w = self.inner.clone();
        vm_exec_on_ui_thread(move |mut env, _| {
            env.call_method(w.as_obj(), "getText", "()Ljava/lang/CharSequence;", &[])?
                .l()?
                .to(&mut env)
        })
        .unwrap()
    }

    pub(crate) fn set_text<S>(&mut self, text: S)
    where
        S: AsRef<str>,
    {
        let text = text.as_ref().to_owned();
        let w = self.inner.clone();
        vm_exec_on_ui_thread(move |mut env, _| {
            let text = env.new_string(&text)?;
            env.call_method(
                w.as_obj(),
                "setText",
                "(Ljava/lang/CharSequence;)V",
                &[(&text).into()],
            )?
            .v()
        })
        .unwrap();
    }
}

impl From<RawWidget> for BaseWidget {
    fn from(value: RawWidget) -> Self {
        Self::new(value)
    }
}

impl Deref for BaseWidget {
    type Target = RawWidget;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
