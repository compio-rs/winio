use winio_handle::{AsMenuBar, AsWindow, BorrowedMenuBar};
use winui3::Microsoft::UI::Xaml::{Controls as MUXC, Visibility};

use crate::Result;

#[derive(Debug)]
pub struct MenuBar {
    element: MUXC::MenuBar,
}

impl MenuBar {
    pub fn new(parent: impl AsWindow) -> Result<Self> {
        let window = parent.as_window().as_winui().clone();
        let element = MUXC::MenuBar::new()?;
        crate::set_menu_bar(&window, element.clone())?;
        Ok(Self { element })
    }

    pub fn is_visible(&self) -> Result<bool> {
        Ok(self.element.Visibility()? == Visibility::Visible)
    }

    pub fn set_visible(&mut self, visible: bool) -> Result<()> {
        self.element.SetVisibility(if visible {
            Visibility::Visible
        } else {
            Visibility::Collapsed
        })?;
        Ok(())
    }
}

impl Drop for MenuBar {
    fn drop(&mut self) {
        crate::remove_menu_bar(&self.element);
    }
}

impl AsMenuBar for MenuBar {
    fn as_menu_bar(&self) -> BorrowedMenuBar<'_> {
        BorrowedMenuBar::winui(&self.element)
    }
}
