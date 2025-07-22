use {
    super::{super::JObjectExt, vm_exec_on_ui_thread},
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

    pub(crate) fn size(&self) -> Size {
        vm_exec_on_ui_thread(|mut env, act| {
            env.call_method(act.as_obj(), "getSize", "()[D", &[])?
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
            )?;

            Ok(())
        })
        .unwrap();
    }

    pub(crate) fn text(&self) -> String {
        vm_exec_on_ui_thread(|mut env, act| {
            env.call_method(act.as_obj(), "getText", "()Ljava/lang/CharSequence;", &[])?
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
            )?;

            Ok(())
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
