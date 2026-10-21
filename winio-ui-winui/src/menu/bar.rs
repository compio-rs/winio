use winio_handle::AsWindow;
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

    pub(crate) fn element(&self) -> &MUXC::MenuBar {
        &self.element
    }
}

impl Drop for MenuBar {
    fn drop(&mut self) {
        crate::remove_menu_bar(&self.element);
    }
}
