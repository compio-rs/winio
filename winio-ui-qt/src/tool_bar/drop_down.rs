use inherit_methods_macro::inherit_methods;
use winio_handle::{AsMenuContainer, BorrowedMenuContainer};

use crate::{
    Image, Result,
    menu::QMenu,
    tool_bar::{ItemKind, ToolBar, ToolBarItem, ffi},
};

#[derive(Debug)]
pub struct ToolBarDropDownButton {
    item: ToolBarItem,
    menu: *mut QMenu,
}

#[inherit_methods(from = "self.item")]
impl ToolBarDropDownButton {
    pub fn new(parent: &ToolBar) -> Result<Self> {
        let item = ToolBarItem::new(parent, ItemKind::DropDownButton)?;
        let menu = unsafe { ffi::tool_bar_action_menu(item.action())? };
        Ok(Self { item, menu })
    }

    pub fn is_enabled(&self) -> Result<bool>;

    pub fn set_enabled(&mut self, enabled: bool) -> Result<()>;

    pub fn text(&self) -> Result<String>;

    pub fn set_text(&mut self, text: impl AsRef<str>) -> Result<()>;

    pub fn set_icon(&mut self, icon: Option<&Image>) -> Result<()>;

    pub async fn start(&self) -> ! {
        std::future::pending().await
    }
}

impl AsMenuContainer for ToolBarDropDownButton {
    fn as_menu_container(&self) -> BorrowedMenuContainer<'_> {
        unsafe { BorrowedMenuContainer::qt(self.menu) }
    }
}

winio_handle::impl_as_tool_bar_item!(ToolBarDropDownButton, item);
