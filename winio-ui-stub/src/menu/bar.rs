use inherit_methods_macro::inherit_methods;
use winio_handle::AsWindow;

use crate::{Result, menu::MenuHandle, not_impl};

#[derive(Debug)]
pub struct MenuBar {
    handle: MenuHandle,
}

#[inherit_methods(from = "self.handle")]
impl MenuBar {
    pub fn new(_parent: impl AsWindow) -> Result<Self> {
        not_impl()
    }

    pub fn is_visible(&self) -> Result<bool>;

    pub fn set_visible(&mut self, v: bool) -> Result<()>;
}
