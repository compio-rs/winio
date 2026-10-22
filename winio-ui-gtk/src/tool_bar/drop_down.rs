use gtk4::gio;
use inherit_methods_macro::inherit_methods;
use winio_handle::{AsMenuContainer, BorrowedMenuContainer};

use crate::{
    Image, Result,
    tool_bar::{ToolBar, ToolBarItem},
};

#[derive(Debug)]
pub struct ToolBarDropDownButton {
    item: ToolBarItem,
    menu: gio::Menu,
}

#[inherit_methods(from = "self.item")]
impl ToolBarDropDownButton {
    pub fn new(parent: &ToolBar) -> Result<Self> {
        let menu = gio::Menu::new();
        let item = ToolBarItem::new_menu_button(parent, &menu)?;
        Ok(Self { item, menu })
    }

    pub fn is_enabled(&self) -> Result<bool>;

    pub fn set_enabled(&mut self, enabled: bool) -> Result<()>;

    pub fn text(&self) -> Result<String>;

    pub fn set_text(&mut self, text: impl AsRef<str>) -> Result<()>;

    pub fn set_icon(&mut self, icon: Option<&Image>) -> Result<()>;

    pub async fn wait_click(&self) {
        self.item.wait_click().await;
    }
}

impl AsMenuContainer for ToolBarDropDownButton {
    fn as_menu_container(&self) -> BorrowedMenuContainer<'_> {
        BorrowedMenuContainer::gtk(&self.menu)
    }
}

winio_handle::impl_as_tool_bar_item!(ToolBarDropDownButton, item);
