use inherit_methods_macro::inherit_methods;
use winio_handle::AsWindow;

use crate::{
    Error, Result,
    tool_bar::ffi,
    widgets::{QWidget, Widget, impl_static_cast},
};

#[derive(Debug)]
pub struct ToolBar {
    widget: Widget<ffi::QToolBar>,
}

#[inherit_methods(from = "self.widget")]
impl ToolBar {
    pub fn new(parent: impl AsWindow) -> Result<Self> {
        let window = parent.as_window().as_qt::<QWidget>();
        if unsafe { ffi::tool_bar_exists(window) } {
            return Err(Error::ToolBarExists);
        }
        let widget = unsafe { ffi::new_tool_bar(window)? };
        let mut widget = Widget::new(widget)?;
        widget.set_visible(true)?;
        Ok(Self { widget })
    }

    pub fn is_visible(&self) -> Result<bool>;

    pub fn set_visible(&mut self, visible: bool) -> Result<()>;

    pub(crate) fn toolbar(&self) -> *mut ffi::QToolBar {
        self.widget.as_ref() as *const _ as *mut _
    }
}

impl_static_cast!(ffi::QToolBar, QWidget);
