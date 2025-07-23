use crate::vm_exec_on_ui_thread;
use {
    super::BaseWidget,
    inherit_methods_macro::inherit_methods,
    winio_handle::{AsWindow, impl_as_widget},
    winio_primitive::{HAlign, Point, Size},
};

#[derive(Debug)]
pub struct Edit {
    inner: BaseWidget,
}

#[inherit_methods(from = "self.inner")]
impl Edit {
    const WIDGET_CLASS: &'static str = "rs/compio/winio/Edit";

    pub async fn wait_change(&self) {
        todo!()
    }

    pub fn text(&self) -> String;

    pub fn set_text<S>(&mut self, _text: S)
    where
        S: AsRef<str>;

    pub fn is_password(&self) -> bool {
        let w = self.inner.clone();
        vm_exec_on_ui_thread(move |mut env, _| {
            env.call_method(w.as_obj(), "isPassword", "()[D", &[])?.z()
        })
        .unwrap()
    }

    pub fn set_password(&mut self, password: bool) {
        let w = self.inner.clone();
        vm_exec_on_ui_thread(move |mut env, _| {
            env.call_method(w.as_obj(), "setPassword", "(Z)V", &[password.into()])?
                .v()
        })
        .unwrap();
    }

    //noinspection SpellCheckingInspection
    pub fn halign(&self) -> HAlign;

    //noinspection SpellCheckingInspection
    pub fn set_halign(&mut self, align: HAlign);

    pub fn is_visible(&self) -> bool;

    pub fn set_visible(&mut self, visible: bool);

    pub fn is_enabled(&self) -> bool {
        todo!()
    }

    pub fn set_enabled(&mut self, _v: bool) {
        todo!()
    }

    pub fn loc(&self) -> Point;

    pub fn set_loc(&mut self, p: Point);

    pub fn size(&self) -> Size;

    pub fn set_size(&mut self, v: Size);

    pub fn preferred_size(&self) -> Size;

    pub fn new<W>(parent: W) -> Self
    where
        W: AsWindow,
    {
        BaseWidget::create(parent.as_window(), Self::WIDGET_CLASS)
    }
}

impl From<BaseWidget> for Edit {
    fn from(value: BaseWidget) -> Self {
        Self { inner: value }
    }
}

impl_as_widget!(Edit, inner);
