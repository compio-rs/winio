use winio_handle::{AsMenuBar, AsWindow, BorrowedMenuBar};
use winui3::Microsoft::UI::Xaml::Controls as MUXC;

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
