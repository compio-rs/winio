use {
    super::{
        super::{define_event, recv_event},
        BaseWidget,
    },
    inherit_methods_macro::inherit_methods,
    winio_handle::{AsWindow, impl_as_widget},
    winio_primitive::{Point, Size},
};

define_event!(
    WAIT_FOR_CHECK_BOX_CLICKING,
    Java_rs_compio_winio_CheckBox_on_1clicked
);

#[derive(Debug)]
pub struct CheckBox {
    inner: BaseWidget,
}

//noinspection SpellCheckingInspection
#[inherit_methods(from = "self.inner")]
impl CheckBox {
    const WIDGET_CLASS: &'static str = "rs/compio/winio/CheckBox";

    pub async fn wait_click(&self) {
        recv_event!(self, WAIT_FOR_CHECK_BOX_CLICKING)
    }

    pub fn text(&self) -> String;

    pub fn set_text<S>(&self, _text: S)
    where
        S: AsRef<str>;

    pub fn is_checked(&self) -> bool;

    pub fn set_checked(&self, _v: bool);

    pub fn is_visible(&self) -> bool;

    pub fn set_visible(&self, visible: bool);

    pub fn loc(&self) -> Point;

    pub fn set_loc(&self, p: Point);

    pub fn size(&self) -> Size;

    pub fn set_size(&self, v: Size);

    pub fn preferred_size(&self) -> Size;

    pub fn is_enabled(&self) -> bool;

    pub fn set_enabled(&self, enabled: bool);

    pub fn new<W>(parent: W) -> Self
    where
        W: AsWindow,
    {
        BaseWidget::create(parent.as_window(), Self::WIDGET_CLASS)
    }
}

impl From<BaseWidget> for CheckBox {
    fn from(value: BaseWidget) -> Self {
        Self { inner: value }
    }
}

impl_as_widget!(CheckBox, inner);
