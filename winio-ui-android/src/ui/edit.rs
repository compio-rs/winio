use {
    super::{
        super::{JObjectExt, define_event, recv_event},
        BaseWidget, vm_exec_on_ui_thread,
    },
    inherit_methods_macro::inherit_methods,
    winio_handle::{AsWindow, impl_as_widget},
    winio_primitive::{HAlign, Point, Size},
};

define_event!(
    WAIT_FOR_EDIT_CHANGING,
    Java_rs_compio_winio_Edit_on_1changed
);

#[derive(Debug)]
pub struct Edit {
    inner: BaseWidget,
}

//noinspection SpellCheckingInspection
#[inherit_methods(from = "self.inner")]
impl Edit {
    const WIDGET_CLASS: &'static str = "rs/compio/winio/Edit";

    pub async fn wait_change(&self) {
        recv_event!(self, WAIT_FOR_EDIT_CHANGING)
    }

    pub fn text(&self) -> String {
        let w = self.inner.clone();
        vm_exec_on_ui_thread(move |mut env, _| {
            env.call_method(
                w.as_obj(),
                "getTextString",
                "()Ljava/lang/CharSequence;",
                &[],
            )?
            .l()?
            .to(&mut env)
        })
        .unwrap()
    }

    pub fn set_text<S>(&self, _text: S)
    where
        S: AsRef<str>;

    pub fn is_password(&self) -> bool {
        let w = self.inner.clone();
        vm_exec_on_ui_thread(move |mut env, _| {
            env.call_method(w.as_obj(), "isPassword", "()[D", &[])?.z()
        })
        .unwrap()
    }

    pub fn set_password(&self, password: bool) {
        let w = self.inner.clone();
        vm_exec_on_ui_thread(move |mut env, _| {
            env.call_method(w.as_obj(), "setPassword", "(Z)V", &[password.into()])?
                .v()
        })
        .unwrap();
    }

    pub fn halign(&self) -> HAlign;

    pub fn set_halign(&self, align: HAlign);

    pub fn is_visible(&self) -> bool;

    pub fn set_visible(&self, visible: bool);

    pub fn is_enabled(&self) -> bool;

    pub fn set_enabled(&self, enabled: bool);

    pub fn loc(&self) -> Point;

    pub fn set_loc(&self, p: Point);

    pub fn size(&self) -> Size;

    pub fn set_size(&self, v: Size);

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
