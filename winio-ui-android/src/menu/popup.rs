use winio_handle::AsMenuContainer;

use crate::{Result, menu::SubMenuItem, vm_exec};

#[derive(Debug)]
pub struct PopupMenuItem {
    inner: SubMenuItem,
}

impl PopupMenuItem {
    pub fn new(parent: impl AsMenuContainer) -> Result<Self> {
        vm_exec(|env| {
            let parent = super::container_menu(env, parent.as_menu_container())?;
            Ok(Self {
                inner: SubMenuItem::new(env, &parent)?,
            })
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

winio_handle::impl_as_menu_container!(PopupMenuItem, inner);
winio_handle::impl_as_menu_item!(PopupMenuItem, inner);
