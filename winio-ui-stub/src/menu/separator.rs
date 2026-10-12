use winio_handle::AsMenuContainer;

use crate::{Result, menu::MenuHandle, not_impl};

#[derive(Debug)]
pub struct SeparatorMenuItem {
    handle: MenuHandle,
}

impl SeparatorMenuItem {
    pub fn new(_parent: impl AsMenuContainer) -> Result<Self> {
        not_impl()
    }
}

winio_handle::impl_as_menu_item!(SeparatorMenuItem, handle);
