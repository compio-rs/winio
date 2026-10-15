use winio_handle::AsMenuContainer;

use crate::{
    Result,
    menu::{MenuNodeRef, NodeKind},
};

#[derive(Debug)]
pub struct SeparatorMenuItem {
    node: MenuNodeRef,
}

impl SeparatorMenuItem {
    pub fn new(parent: impl AsMenuContainer) -> Result<Self> {
        let parent = parent.as_menu_container().as_ui_kit();
        Ok(Self {
            node: MenuNodeRef::child(parent, NodeKind::Separator),
        })
    }
}

winio_handle::impl_as_menu_item!(SeparatorMenuItem, node);
