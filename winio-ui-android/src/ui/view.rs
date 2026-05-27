use inherit_methods_macro::inherit_methods;
use winio_handle::AsContainer;
use winio_primitive::{Point, Size};

use crate::{BaseWidget, Result};

#[derive(Debug)]
pub struct View {
    inner: BaseWidget,
}

#[inherit_methods(from = "self.inner")]
impl View {
    const WIDGET_CLASS: &'static str = "android/widget/FrameLayout";

    pub fn new(parent: impl AsContainer) -> Result<Self> {
        Ok(Self {
            inner: BaseWidget::new(parent.as_container(), Self::WIDGET_CLASS)?,
        })
    }

    pub fn client_size(&self) -> Result<Size> {
        self.size()
    }

    pub fn is_visible(&self) -> Result<bool>;

    pub fn set_visible(&mut self, visible: bool) -> Result<()>;

    pub fn loc(&self) -> Result<Point>;

    pub fn set_loc(&mut self, p: Point) -> Result<()>;

    pub fn size(&self) -> Result<Size>;

    pub fn set_size(&mut self, size: Size) -> Result<()>;

    pub fn text(&self) -> Result<String>;

    pub fn set_text(&mut self, text: impl AsRef<str>) -> Result<()>;
}

winio_handle::impl_as_widget!(View, inner);
winio_handle::impl_as_container!(View, inner);
