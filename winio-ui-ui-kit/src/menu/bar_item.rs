use std::rc::Rc;

use crate::{
    Result,
    menu::{MenuBar, MenuNodeRef, system::MenuBarData},
};

#[derive(Debug)]
pub struct MenuBarItem {
    node: MenuNodeRef,
    data: Rc<MenuBarData>,
}

impl MenuBarItem {
    pub fn new(parent: &MenuBar) -> Result<Self> {
        let data = parent.data().clone();
        let node = MenuNodeRef::root();
        data.add_item(node.key());
        Ok(Self { node, data })
    }

    pub fn is_enabled(&self) -> Result<bool> {
        Ok(self.node.is_enabled())
    }

    pub fn set_enabled(&mut self, enabled: bool) -> Result<()> {
        self.node.set_enabled(enabled);
        Ok(())
    }

    pub fn text(&self) -> Result<String> {
        Ok(self.node.text())
    }

    pub fn set_text(&mut self, s: impl AsRef<str>) -> Result<()> {
        self.node.set_text(s);
        Ok(())
    }
}

impl Drop for MenuBarItem {
    fn drop(&mut self) {
        self.data.remove_item(self.node.key());
    }
}

winio_handle::impl_as_menu_container!(MenuBarItem, node);
