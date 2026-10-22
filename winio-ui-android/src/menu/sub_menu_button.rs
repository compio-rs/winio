use jni::Env;
use winio_handle::{AsToolBarItem, BorrowedToolBarItem};

use crate::{Image, Result, java::android::view::Menu as JMenu, menu::SubMenuItem, vm_exec};

#[derive(Debug)]
pub(crate) struct SubMenuButton {
    inner: SubMenuItem,
}

impl SubMenuButton {
    pub(crate) fn new(env: &mut Env<'_>, menu: &JMenu<'_>) -> Result<Self> {
        let inner = SubMenuItem::new(env, menu)?;
        inner.set_show_as_action(env)?;
        Ok(Self { inner })
    }

    pub(crate) fn is_enabled(&self) -> Result<bool> {
        vm_exec(|env| self.inner.is_enabled(env))
    }

    pub(crate) fn set_enabled(&mut self, enabled: bool) -> Result<()> {
        vm_exec(|env| self.inner.set_enabled(env, enabled))
    }

    pub(crate) fn text(&self) -> Result<String> {
        vm_exec(|env| self.inner.text(env))
    }

    pub(crate) fn set_text(&mut self, s: impl AsRef<str>) -> Result<()> {
        vm_exec(|env| self.inner.set_text(env, s))
    }

    pub(crate) fn set_icon(&mut self, icon: Option<&Image>) -> Result<()> {
        vm_exec(|env| self.inner.set_icon(env, icon))
    }
}

winio_handle::impl_as_menu_container!(SubMenuButton, inner);

impl AsToolBarItem for SubMenuButton {
    fn as_tool_bar_item(&self) -> BorrowedToolBarItem<'_> {
        // SAFETY: the menu item is valid for the lifetime of `self`.
        unsafe { BorrowedToolBarItem::android(self.inner.item()) }
    }
}
