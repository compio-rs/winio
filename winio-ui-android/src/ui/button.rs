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
    WAIT_FOR_BUTTON_CLICKING,
    Java_rs_compio_winio_Button_on_1clicked
);

#[derive(Debug)]
pub struct Button {
    inner: BaseWidget,
}

#[inherit_methods(from = "self.inner")]
impl Button {
    const WIDGET_CLASS: &'static str = "rs/compio/winio/Button";

    pub async fn wait_click(&self) {
        recv_event!(self, WAIT_FOR_BUTTON_CLICKING)
    }

    pub fn text(&self) -> String;

    pub fn set_text<S>(&mut self, _text: S)
    where
        S: AsRef<str>;

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

impl From<BaseWidget> for Button {
    fn from(value: BaseWidget) -> Self {
        Self { inner: value }
    }
}

impl_as_widget!(Button, inner);
