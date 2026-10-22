use winio_handle::{AsToolBarItem, BorrowedMenuContainer, BorrowedToolBarItem};

use crate::{
    Image, Result,
    menu::{LeafItem, LeafKind, SubMenuItem},
    tool_bar::ToolBar,
    vm_exec,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ItemKind {
    Button,
    CheckBox,
    RadioButton,
}

#[derive(Debug)]
pub(crate) struct ToolBarItem {
    item: LeafItem,
    kind: ItemKind,
}

impl ToolBarItem {
    pub(crate) fn new(parent: &ToolBar, kind: ItemKind) -> Result<Self> {
        vm_exec(|env| {
            let leaf_kind = match kind {
                ItemKind::Button => LeafKind::Plain,
                ItemKind::CheckBox | ItemKind::RadioButton => LeafKind::Check,
            };
            let item = LeafItem::new(
                env,
                unsafe { BorrowedMenuContainer::android(parent.menu()) },
                leaf_kind,
            )?;
            item.set_show_as_action(env)?;
            Ok(Self { item, kind })
        })
    }

    pub(crate) fn is_enabled(&self) -> Result<bool> {
        vm_exec(|env| self.item.is_enabled(env))
    }

    pub(crate) fn set_enabled(&self, enabled: bool) -> Result<()> {
        vm_exec(|env| self.item.set_enabled(env, enabled))
    }

    pub(crate) fn text(&self) -> Result<String> {
        vm_exec(|env| self.item.text(env))
    }

    pub(crate) fn set_text(&self, s: impl AsRef<str>) -> Result<()> {
        vm_exec(|env| self.item.set_text(env, s))
    }

    pub(crate) fn set_icon(&self, icon: Option<&Image>) -> Result<()> {
        vm_exec(|env| self.item.set_icon(env, icon))
    }

    pub(crate) fn is_checked(&self) -> Result<bool> {
        vm_exec(|env| self.item.is_checked(env))
    }

    pub(crate) fn set_checked(&self, checked: bool) -> Result<()> {
        vm_exec(|env| self.item.set_checked(env, checked))
    }

    pub(crate) async fn wait_click(&self) {
        self.item.wait_click().await;
        match self.kind {
            ItemKind::Button => {}
            ItemKind::CheckBox => {
                let _ = vm_exec(|env| self.item.toggle_checked(env));
            }
            ItemKind::RadioButton => {
                // A radio button must stay checked when it is clicked again;
                // the radio group updates the other items afterwards.
                let _ = vm_exec(|env| self.item.set_checked(env, true));
            }
        }
    }
}

impl AsToolBarItem for ToolBarItem {
    fn as_tool_bar_item(&self) -> BorrowedToolBarItem<'_> {
        // SAFETY: the menu item is valid for the lifetime of `self`.
        unsafe { BorrowedToolBarItem::android(self.item.item()) }
    }
}

impl AsToolBarItem for SubMenuItem {
    fn as_tool_bar_item(&self) -> BorrowedToolBarItem<'_> {
        // SAFETY: the menu item is valid for the lifetime of `self`.
        unsafe { BorrowedToolBarItem::android(self.item()) }
    }
}
