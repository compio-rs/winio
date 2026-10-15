use winio_handle::AsMenuContainer;

use crate::{
    Result,
    menu::{MenuNodeRef, NodeKind},
};

#[derive(Debug)]
pub struct PopupMenuItem {
    node: MenuNodeRef,
}

impl PopupMenuItem {
    pub fn new(parent: impl AsMenuContainer) -> Result<Self> {
        let parent = parent.as_menu_container().as_ui_kit();
        Ok(Self {
            node: MenuNodeRef::child(parent, NodeKind::Popup),
        })
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

winio_handle::impl_as_menu_container!(PopupMenuItem, node);
winio_handle::impl_as_menu_item!(PopupMenuItem, node);
