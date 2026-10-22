use windows_core::{HSTRING, Interface, Weak};
use winio_handle::{AsMenuContainer, BorrowedMenuContainer};
use winui3::Microsoft::UI::Xaml::Controls as MUXC;

use crate::{Result, menu::MenuBar};

pub struct MenuBarItem {
    parent: Weak<MUXC::MenuBar>,
    element: MUXC::MenuBarItem,
}

impl std::fmt::Debug for MenuBarItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MenuBarItem").finish_non_exhaustive()
    }
}

impl Drop for MenuBarItem {
    fn drop(&mut self) {
        let Some(parent) = self.parent.upgrade() else {
            return;
        };
        let Ok(items) = parent.Items() else {
            return;
        };
        let mut index = 0;
        if let Ok(true) = items.IndexOf(&self.element, &mut index) {
            let _ = items.RemoveAt(index);
        }
    }
}

impl MenuBarItem {
    pub fn new(parent: &MenuBar) -> Result<Self> {
        let element = MUXC::MenuBarItem::new()?;
        let parent = parent.element();
        parent.Items()?.Append(&element)?;
        Ok(Self {
            parent: parent.downgrade()?,
            element,
        })
    }

    pub fn is_enabled(&self) -> Result<bool> {
        self.element.IsEnabled()
    }

    pub fn set_enabled(&mut self, enabled: bool) -> Result<()> {
        self.element.SetIsEnabled(enabled)?;
        Ok(())
    }

    pub fn text(&self) -> Result<String> {
        Ok(self.element.Title()?.to_string_lossy())
    }

    pub fn set_text(&mut self, text: impl AsRef<str>) -> Result<()> {
        self.element.SetTitle(&HSTRING::from(text.as_ref()))?;
        Ok(())
    }
}

impl AsMenuContainer for MenuBarItem {
    fn as_menu_container(&self) -> BorrowedMenuContainer<'_> {
        BorrowedMenuContainer::winui_menu_bar(&self.element)
    }
}
