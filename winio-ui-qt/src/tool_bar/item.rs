use inherit_methods_macro::inherit_methods;
use winio_handle::{AsToolBarItem, BorrowedToolBarItem};

use crate::{
    Image, Result,
    menu::{ActionClick, ActionHandle, QAction},
    tool_bar::{ToolBar, ffi},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ItemKind {
    Button,
    CheckBox,
    RadioButton,
    DropDownButton,
    Separator,
}

#[derive(Debug)]
pub(crate) struct ToolBarItem {
    action: ActionHandle,
    click: ActionClick,
}

#[inherit_methods(from = "self.action")]
impl ToolBarItem {
    pub(crate) fn new(parent: &ToolBar, kind: ItemKind) -> Result<Self> {
        let action = unsafe {
            match kind {
                ItemKind::Separator => ffi::tool_bar_add_separator(parent.toolbar())?,
                ItemKind::DropDownButton => ffi::tool_bar_add_menu_action(parent.toolbar())?,
                _ => ffi::tool_bar_add_action(parent.toolbar())?,
            }
        };
        let mut action = ActionHandle::new(action);
        if matches!(kind, ItemKind::CheckBox | ItemKind::RadioButton) {
            action.set_checkable(true)?;
        }
        let click = ActionClick::new(action.action_ptr())?;
        Ok(Self { action, click })
    }

    pub(crate) fn action(&self) -> *mut QAction {
        self.action.action_ptr()
    }

    pub(crate) fn is_enabled(&self) -> Result<bool>;

    pub(crate) fn set_enabled(&mut self, enabled: bool) -> Result<()>;

    pub(crate) fn text(&self) -> Result<String>;

    pub(crate) fn set_text(&mut self, text: impl AsRef<str>) -> Result<()>;

    pub(crate) fn is_checked(&self) -> Result<bool>;

    pub(crate) fn set_checked(&mut self, checked: bool) -> Result<()>;

    pub(crate) fn set_icon(&mut self, icon: Option<&Image>) -> Result<()>;

    pub(crate) async fn wait_click(&self) {
        self.click.wait_click().await;
    }
}

impl AsToolBarItem for ToolBarItem {
    fn as_tool_bar_item(&self) -> BorrowedToolBarItem<'_> {
        unsafe { BorrowedToolBarItem::qt(self.action.action_ptr()) }
    }
}
