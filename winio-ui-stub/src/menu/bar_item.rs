use inherit_methods_macro::inherit_methods;
use winio_handle::AsMenuBar;

use crate::{Result, menu::MenuHandle, not_impl};

#[derive(Debug)]
pub struct MenuBarItem {
    handle: MenuHandle,
}

#[inherit_methods(from = "self.handle")]
impl MenuBarItem {
    pub fn new(_parent: impl AsMenuBar) -> Result<Self> {
        not_impl()
    }

    pub fn is_enabled(&self) -> Result<bool>;

    pub fn set_enabled(&mut self, enabled: bool) -> Result<()>;

    pub fn text(&self) -> Result<String>;

    pub fn set_text(&mut self, s: impl AsRef<str>) -> Result<()>;
}

winio_handle::impl_as_menu_container!(MenuBarItem, handle);
