use winio_handle::AsMenuContainer;

use crate::{
    Result,
    menu::{MenuNodeRef, NodeKind},
};

#[derive(Debug)]
pub struct CheckMenuItem {
    node: MenuNodeRef,
}

impl CheckMenuItem {
    pub fn new(parent: impl AsMenuContainer) -> Result<Self> {
        let parent = parent.as_menu_container().as_ui_kit();
        Ok(Self {
            node: MenuNodeRef::child(parent, NodeKind::Check),
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

    pub fn is_checked(&self) -> Result<bool> {
        Ok(self.node.is_checked())
    }

    pub fn set_checked(&mut self, checked: bool) -> Result<()> {
        self.node.set_checked(checked);
        Ok(())
    }

    pub async fn wait_click(&self) {
        self.node.wait_click().await;
        self.node.toggle_checked();
    }
}

winio_handle::impl_as_menu_item!(CheckMenuItem, node);
