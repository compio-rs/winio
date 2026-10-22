use inherit_methods_macro::inherit_methods;
use jni::refs::Global;
use winio_handle::AsWindow;

use crate::{
    Result,
    bar::{BarBase, BarKind},
    java::android::view::Menu as JMenu,
};

#[derive(Debug)]
pub struct ToolBar {
    inner: BarBase,
}

#[inherit_methods(from = "self.inner")]
impl ToolBar {
    pub fn new(parent: impl AsWindow) -> Result<Self> {
        Ok(Self {
            inner: BarBase::new(parent, BarKind::ToolBar)?,
        })
    }

    pub fn is_visible(&self) -> Result<bool>;

    pub fn set_visible(&mut self, visible: bool) -> Result<()>;

    pub(crate) fn menu(&self) -> &Global<JMenu<'static>> {
        self.inner.menu()
    }
}
