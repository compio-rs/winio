use inherit_methods_macro::inherit_methods;

use super::{ItemKind, ToolBarItem, bar::ToolBar};
use crate::{
    Image, Result,
    menu::{OwnedMenu, set_menu_owner},
};

#[derive(Debug)]
pub struct ToolBarDropDownButton {
    item: ToolBarItem,
    menu: OwnedMenu,
}

#[inherit_methods(from = "self.item")]
impl ToolBarDropDownButton {
    pub fn new(parent: &ToolBar) -> Result<Self> {
        let item = ToolBarItem::new(parent, ItemKind::DropDownButton)?;
        let menu = OwnedMenu::new_popup()?;
        set_menu_owner(&menu, parent.data().handle.parent());
        Ok(Self { item, menu })
    }

    pub fn is_enabled(&self) -> Result<bool>;

    pub fn set_enabled(&self, enabled: bool) -> Result<()>;

    pub fn text(&self) -> Result<String>;

    pub fn set_text(&self, text: impl AsRef<str>) -> Result<()>;

    pub fn set_icon(&self, icon: Option<&Image>) -> Result<()>;

    pub async fn start(&self) -> ! {
        loop {
            self.item.wait_drop_down().await;
            self.item.show_drop_down(&self.menu);
        }
    }
}

winio_handle::impl_as_menu_container!(ToolBarDropDownButton, menu);
winio_handle::impl_as_tool_bar_item!(ToolBarDropDownButton, item);
