use inherit_methods_macro::inherit_methods;
use winio_handle::AsMenuContainer;

use crate::{Image, Result, menu::MenuHandle, not_impl};

#[derive(Debug)]
pub struct MenuItem {
    handle: MenuHandle,
}

#[inherit_methods(from = "self.handle")]
impl MenuItem {
    pub fn new(_parent: impl AsMenuContainer) -> Result<Self> {
        not_impl()
    }

    pub fn is_enabled(&self) -> Result<bool>;

    pub fn set_enabled(&mut self, enabled: bool) -> Result<()>;

    pub fn text(&self) -> Result<String>;

    pub fn set_text(&mut self, s: impl AsRef<str>) -> Result<()>;

    pub fn set_icon(&mut self, icon: Option<&Image>) -> Result<()>;

    pub async fn wait_click(&self) {
        not_impl()
    }
}

winio_handle::impl_as_menu_item!(MenuItem, handle);
