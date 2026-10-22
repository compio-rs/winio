use inherit_methods_macro::inherit_methods;

use crate::{
    Result,
    menu::{MenuBar, SubMenuButton},
    vm_exec,
};

#[derive(Debug)]
pub struct MenuBarItem {
    inner: SubMenuButton,
}

#[inherit_methods(from = "self.inner")]
impl MenuBarItem {
    pub fn new(parent: &MenuBar) -> Result<Self> {
        vm_exec(|env| {
            Ok(Self {
                inner: SubMenuButton::new(env, parent.menu())?,
            })
        })
    }

    pub fn is_enabled(&self) -> Result<bool>;

    pub fn set_enabled(&mut self, enabled: bool) -> Result<()>;

    pub fn text(&self) -> Result<String>;

    pub fn set_text(&mut self, s: impl AsRef<str>) -> Result<()>;
}

winio_handle::impl_as_menu_container!(MenuBarItem, inner);
