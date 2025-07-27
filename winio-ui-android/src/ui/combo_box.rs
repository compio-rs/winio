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
    WAIT_FOR_COMBO_BOX_CHANGING,
    Java_rs_compio_winio_ComboBox_on_1change
);
define_event!(
    WAIT_FOR_COMBO_BOX_SELECTING,
    Java_rs_compio_winio_ComboBox_on_1select
);

#[derive(Debug)]
pub struct ComboBox {
    inner: BaseWidget,
}

//noinspection SpellCheckingInspection
#[inherit_methods(from = "self.inner")]
impl ComboBox {
    const WIDGET_CLASS: &'static str = "rs/compio/winio/ComboBox";

    pub async fn wait_change(&self) {
        recv_event!(self, WAIT_FOR_COMBO_BOX_CHANGING)
    }

    pub async fn wait_select(&self) {
        recv_event!(self, WAIT_FOR_COMBO_BOX_SELECTING)
    }

    pub fn selection(&self) -> Option<usize>;

    pub fn set_selection(&self, i: Option<usize>);

    pub fn len(&self) -> usize;

    pub fn is_editable(&self) -> bool;

    pub fn set_editable(&self, editable: bool);

    pub fn is_empty(&self) -> bool;

    pub fn clear(&self);

    pub fn get(&self, i: usize) -> String;

    pub fn set<S>(&self, i: usize, item: S)
    where
        S: AsRef<str>;

    pub fn insert<S>(&self, i: usize, item: S)
    where
        S: AsRef<str>;

    pub fn remove(&self, i: usize);

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

impl From<BaseWidget> for ComboBox {
    fn from(value: BaseWidget) -> Self {
        Self { inner: value }
    }
}

impl_as_widget!(ComboBox, inner);
