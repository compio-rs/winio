use inherit_methods_macro::inherit_methods;
use winio_handle::AsMenuBar;

use crate::{Result, menu::PopupMenuItemBase};

#[derive(Debug)]
pub struct MenuBarItem {
    base: PopupMenuItemBase,
}

#[inherit_methods(from = "self.base")]
impl MenuBarItem {
    pub fn new(parent: impl AsMenuBar) -> Result<Self> {
        let parent = parent.as_menu_bar().to_gtk();

        let base = PopupMenuItemBase::new(parent)?;
        Ok(Self { base })
    }

    pub fn is_enabled(&self) -> Result<bool>;

    pub fn set_enabled(&mut self, enabled: bool) -> Result<()>;

    pub fn text(&self) -> Result<String>;

    pub fn set_text(&mut self, s: impl AsRef<str>) -> Result<()>;
}

winio_handle::impl_as_menu_container!(MenuBarItem, base);
