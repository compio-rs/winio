use windows_core::{HSTRING, Interface};
use winio_handle::{AsMenuContainer, AsMenuItem, BorrowedMenuContainer, BorrowedMenuItem};
use winui3::Microsoft::UI::Xaml::Controls as MUXC;

use crate::{Image, Result, menu::append_to_container};

#[derive(Debug)]
pub struct PopupMenuItem {
    element: MUXC::MenuFlyoutSubItem,
    base: MUXC::MenuFlyoutItemBase,
}

impl PopupMenuItem {
    pub fn new(parent: impl AsMenuContainer) -> Result<Self> {
        let element = MUXC::MenuFlyoutSubItem::new()?;
        let base = element.cast()?;
        append_to_container(parent.as_menu_container(), &base)?;
        Ok(Self { element, base })
    }

    pub fn is_enabled(&self) -> Result<bool> {
        self.element.IsEnabled()
    }

    pub fn set_enabled(&mut self, enabled: bool) -> Result<()> {
        self.element.SetIsEnabled(enabled)?;
        Ok(())
    }

    pub fn text(&self) -> Result<String> {
        Ok(self.element.Text()?.to_string_lossy())
    }

    pub fn set_text(&mut self, text: impl AsRef<str>) -> Result<()> {
        self.element.SetText(&HSTRING::from(text.as_ref()))?;
        Ok(())
    }

    pub fn set_icon(&mut self, icon: Option<&Image>) -> Result<()> {
        let icon = match icon {
            Some(icon) => {
                let image_icon = MUXC::ImageIcon::new()?;
                image_icon.SetSource(icon.as_ref())?;
                Some(image_icon.cast::<MUXC::IconElement>()?)
            }
            None => None,
        };
        self.element.SetIcon(icon.as_ref())?;
        Ok(())
    }
}

impl AsMenuContainer for PopupMenuItem {
    fn as_menu_container(&self) -> BorrowedMenuContainer<'_> {
        BorrowedMenuContainer::winui_menu_item(&self.element)
    }
}

impl AsMenuItem for PopupMenuItem {
    fn as_menu_item(&self) -> BorrowedMenuItem<'_> {
        BorrowedMenuItem::winui(&self.base)
    }
}
