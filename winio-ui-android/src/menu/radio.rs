use winio_handle::AsMenuContainer;

use crate::{
    Result,
    menu::{LeafItem, LeafKind},
    vm_exec,
};

#[derive(Debug)]
pub struct RadioMenuItem {
    inner: LeafItem,
}

impl RadioMenuItem {
    pub fn new(parent: impl AsMenuContainer) -> Result<Self> {
        vm_exec(|env| {
            Ok(Self {
                inner: LeafItem::new(env, parent.as_menu_container(), LeafKind::Radio)?,
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

    pub fn is_checked(&self) -> Result<bool> {
        vm_exec(|env| self.inner.is_checked(env))
    }

    pub fn set_checked(&mut self, checked: bool) -> Result<()> {
        vm_exec(|env| self.inner.set_checked(env, checked))
    }

    pub async fn wait_click(&self) {
        self.inner.wait_click().await;
        let _ = vm_exec(|env| self.inner.set_checked(env, true));
    }
}

winio_handle::impl_as_menu_item!(RadioMenuItem, inner);
