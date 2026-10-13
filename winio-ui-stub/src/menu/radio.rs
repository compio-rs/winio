use inherit_methods_macro::inherit_methods;
use winio_handle::AsMenuContainer;

use crate::{Result, menu::MenuHandle, not_impl};

#[derive(Debug)]
pub struct RadioMenuItem {
    handle: MenuHandle,
}

#[inherit_methods(from = "self.handle")]
impl RadioMenuItem {
    pub fn new(_parent: impl AsMenuContainer) -> Result<Self> {
        not_impl()
    }

    pub fn is_enabled(&self) -> Result<bool>;

    pub fn set_enabled(&mut self, enabled: bool) -> Result<()>;

    pub fn text(&self) -> Result<String>;

    pub fn set_text(&mut self, s: impl AsRef<str>) -> Result<()>;

    pub fn is_checked(&self) -> Result<bool>;

    pub fn set_checked(&mut self, checked: bool) -> Result<()>;

    pub async fn wait_click(&self) {
        not_impl()
    }
}

winio_handle::impl_as_menu_item!(RadioMenuItem, handle);
