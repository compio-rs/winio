use gtk4::gio;
use inherit_methods_macro::inherit_methods;
use winio_handle::{AsMenuContainer, BorrowedMenuContainer};

use crate::{Result, menu::ItemBase};

#[derive(Debug)]
pub(crate) struct PopupMenuItemBase {
    item: ItemBase,
    menu: gio::Menu,
}

#[inherit_methods(from = "self.item")]
impl PopupMenuItemBase {
    pub fn new(parent: &gio::Menu) -> Result<Self> {
        let menu = gio::Menu::new();
        let item = ItemBase::new(parent, None, None, Some(&menu))?;

        Ok(Self { item, menu })
    }

    pub fn is_enabled(&self) -> Result<bool> {
        Ok(true)
    }

    pub fn set_enabled(&mut self, _enabled: bool) -> Result<()> {
        Ok(())
    }

    pub fn text(&self) -> Result<String>;

    pub fn set_text(&mut self, s: impl AsRef<str>) -> Result<()>;
}

impl AsMenuContainer for PopupMenuItemBase {
    fn as_menu_container(&self) -> BorrowedMenuContainer<'_> {
        BorrowedMenuContainer::gtk(&self.menu)
    }
}

winio_handle::impl_as_menu_item!(PopupMenuItemBase, item);

#[derive(Debug)]
pub struct PopupMenuItem {
    base: PopupMenuItemBase,
}

#[inherit_methods(from = "self.base")]
impl PopupMenuItem {
    pub fn new(parent: impl AsMenuContainer) -> Result<Self> {
        let parent = parent.as_menu_container().to_gtk();
        let base = PopupMenuItemBase::new(parent)?;
        Ok(Self { base })
    }

    pub fn is_enabled(&self) -> Result<bool>;

    pub fn set_enabled(&mut self, enabled: bool) -> Result<()>;

    pub fn text(&self) -> Result<String>;

    pub fn set_text(&mut self, s: impl AsRef<str>) -> Result<()>;
}

winio_handle::impl_as_menu_container!(PopupMenuItem, base);
winio_handle::impl_as_menu_item!(PopupMenuItem, base);
