use windows_core::HSTRING;
use winio_handle::{AsMenuContainer, BorrowedMenuContainer};
use winui3::Microsoft::UI::Xaml::Controls as MUXC;

use crate::{Result, menu::MenuBar};

#[derive(Debug)]
pub struct MenuBarItem {
    element: MUXC::MenuBarItem,
}

impl MenuBarItem {
    pub fn new(parent: &MenuBar) -> Result<Self> {
        let element = MUXC::MenuBarItem::new()?;
        parent.element().Items()?.Append(&element)?;
        Ok(Self { element })
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
