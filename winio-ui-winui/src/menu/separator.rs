use windows_core::Interface;
use winio_handle::{AsMenuContainer, AsMenuItem, BorrowedMenuItem};
use winui3::Microsoft::UI::Xaml::Controls as MUXC;

use crate::{
    Result,
    menu::{MenuParent, append_to_container},
};

#[derive(Debug)]
pub struct SeparatorMenuItem {
    base: MUXC::MenuFlyoutItemBase,
    parent: MenuParent,
}

impl Drop for SeparatorMenuItem {
    fn drop(&mut self) {
        let _ = self.parent.remove(&self.base);
    }
}

impl SeparatorMenuItem {
    pub fn new(parent: impl AsMenuContainer) -> Result<Self> {
        let element = MUXC::MenuFlyoutSeparator::new()?;
        let base = element.cast()?;
        let parent = append_to_container(parent.as_menu_container(), &base)?;
        Ok(Self { base, parent })
    }
}

impl AsMenuItem for SeparatorMenuItem {
    fn as_menu_item(&self) -> BorrowedMenuItem<'_> {
        BorrowedMenuItem::winui(&self.base)
    }
}
