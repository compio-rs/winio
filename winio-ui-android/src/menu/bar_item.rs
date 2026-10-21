use crate::{
    Result,
    menu::{MenuBar, SubMenuItem},
    vm_exec,
};

#[derive(Debug)]
pub struct MenuBarItem {
    inner: SubMenuItem,
}

impl MenuBarItem {
    pub fn new(parent: &MenuBar) -> Result<Self> {
        vm_exec(|env| {
            let inner = SubMenuItem::new(env, parent.menu())?;
            inner.set_show_as_action(env)?;
            Ok(Self { inner })
        })
    }

    pub fn is_enabled(&self) -> Result<bool> {
        vm_exec(|env| self.inner.is_enabled(env))
    }

    pub fn set_enabled(&mut self, enabled: bool) -> Result<()> {
        vm_exec(|env| self.inner.set_enabled(env, enabled))
    }

    pub fn text(&self) -> Result<String> {
        vm_exec(|env| self.inner.text(env))
    }

    pub fn set_text(&mut self, s: impl AsRef<str>) -> Result<()> {
        vm_exec(|env| self.inner.set_text(env, s))
    }
}

winio_handle::impl_as_menu_container!(MenuBarItem, inner);
