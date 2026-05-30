use inherit_methods_macro::inherit_methods;
use winio_handle::AsContainer;
use winio_primitive::{Point, Size};

use crate::{BaseWidget, FrameLayout, Result, current_activity, vm_exec};

#[derive(Debug)]
pub struct View {
    inner: BaseWidget<FrameLayout<'static>>,
}

#[inherit_methods(from = "self.inner")]
impl View {
    pub fn new(parent: impl AsContainer) -> Result<Self> {
        vm_exec(|env| {
            let act = current_activity(env)?;
            let widget = FrameLayout::new(env, act)?;
            let inner = BaseWidget::new_with_env(env, parent.as_container(), widget)?;
            Ok(Self { inner })
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
}

winio_handle::impl_as_widget!(View, inner);
winio_handle::impl_as_container!(View, inner);
