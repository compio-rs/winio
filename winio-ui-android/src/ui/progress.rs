use {
    super::BaseWidget,
    inherit_methods_macro::inherit_methods,
    winio_handle::{AsWindow, impl_as_widget},
    winio_primitive::{Point, Size},
};

#[derive(Debug)]
pub struct Progress {
    inner: BaseWidget,
}

//noinspection SpellCheckingInspection
#[inherit_methods(from = "self.inner")]
impl Progress {
    const WIDGET_CLASS: &'static str = "rs/compio/winio/Progress";

    pub fn range(&self) -> (usize, usize);

    pub fn set_range(&self, min: usize, max: usize);

    pub fn pos(&self) -> usize;

    pub fn set_pos(&self, pos: usize);

    pub fn is_indeterminate(&self) -> bool;

    pub fn set_indeterminate(&self, indeterminate: bool);

    pub fn is_visible(&self) -> bool;

    pub fn set_visible(&self, visible: bool);

    pub fn is_enabled(&self) -> bool;

    pub fn set_enabled(&self, enabled: bool);

    pub fn loc(&self) -> Point;

    pub fn set_loc(&self, p: Point);

    pub fn size(&self) -> Size;

    pub fn set_size(&self, v: Size);

    pub fn preferred_size(&self) -> Size;

    pub fn minimum(&self) -> usize;

    pub fn set_minimum(&self, minimum: usize);

    pub fn maximum(&self) -> usize;

    pub fn set_maximum(&self, maximum: usize);

    pub fn new<W>(parent: W) -> Self
    where
        W: AsWindow,
    {
        BaseWidget::create(parent.as_window(), Self::WIDGET_CLASS)
    }
}

impl From<BaseWidget> for Progress {
    fn from(value: BaseWidget) -> Self {
        Self { inner: value }
    }
}

impl_as_widget!(Progress, inner);
