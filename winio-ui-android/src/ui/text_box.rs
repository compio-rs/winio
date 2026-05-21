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
    WAIT_FOR_TEXT_BOX_CHANGING,
    Java_rs_compio_winio_TextBox_on_1changed
);

#[derive(Debug)]
pub struct TextBox {
    inner: BaseWidget,
}

//noinspection SpellCheckingInspection
#[inherit_methods(from = "self.inner")]
impl TextBox {
    const WIDGET_CLASS: &'static str = "rs/compio/winio/TextBox";

    pub async fn wait_change(&self) {
        recv_event!(self, WAIT_FOR_TEXT_BOX_CHANGING)
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

    pub fn halign(&self) -> HAlign;

    pub fn set_halign(&self, _align: HAlign);

    pub fn is_visible(&self) -> bool;

    pub fn set_visible(&self, visible: bool);

    pub fn is_enabled(&self) -> bool;

    pub fn set_enabled(&self, enabled: bool);

    pub fn loc(&self) -> Point;

    pub fn set_loc(&self, p: Point);

    pub fn size(&self) -> Size;

    pub fn set_size(&self, v: Size);

    pub fn min_size(&self) -> Size {
        todo!()
    }

    pub fn preferred_size(&self) -> Size;

    pub fn new<W>(parent: W) -> Self
    where
        W: AsWindow,
    {
        BaseWidget::create(parent.as_window(), Self::WIDGET_CLASS)
    }
}

impl From<BaseWidget> for TextBox {
    fn from(value: BaseWidget) -> Self {
        Self { inner: value }
    }
}

impl_as_widget!(TextBox, inner);
