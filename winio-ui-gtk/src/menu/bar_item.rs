use inherit_methods_macro::inherit_methods;

use crate::{
    Result,
    menu::{MenuBar, PopupMenuItemBase},
};

#[derive(Debug)]
pub struct MenuBarItem {
    base: PopupMenuItemBase,
}

#[inherit_methods(from = "self.base")]
impl MenuBarItem {
    pub fn new(parent: &MenuBar) -> Result<Self> {
        let base = PopupMenuItemBase::new(parent.menu())?;
        Ok(Self { base })
    }

    pub fn is_enabled(&self) -> Result<bool>;

    pub fn set_enabled(&mut self, enabled: bool) -> Result<()>;

    pub fn text(&self) -> Result<String>;

    pub fn set_text(&mut self, s: impl AsRef<str>) -> Result<()>;
}

winio_handle::impl_as_menu_container!(MenuBarItem, base);
