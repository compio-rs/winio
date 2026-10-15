use winio_handle::AsMenuBar;

use crate::{
    Result,
    menu::{SubMenuItem, bar_menu},
    vm_exec,
};

#[derive(Debug)]
pub struct MenuBarItem {
    inner: SubMenuItem,
}

impl MenuBarItem {
    pub fn new(parent: impl AsMenuBar) -> Result<Self> {
        vm_exec(|env| {
            let menu = bar_menu(env, parent.as_menu_bar())?;
            let inner = SubMenuItem::new(env, &menu)?;
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
