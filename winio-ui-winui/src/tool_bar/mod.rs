use windows_core::Interface;
use winio_handle::{AsToolBarItem, BorrowedToolBarItem};
use winui3::Microsoft::UI::Xaml::Controls as MUXC;

use crate::Result;

mod bar;
pub use bar::*;

mod button;
pub use button::*;

mod check_box;
pub use check_box::*;

mod drop_down;
pub use drop_down::*;

mod radio_button;
pub use radio_button::*;

mod separator;
pub use separator::*;

#[derive(Debug)]
pub(crate) struct ToolBarItem {
    handle: MUXC::ICommandBarElement,
}

impl ToolBarItem {
    pub fn new(handle: MUXC::ICommandBarElement) -> Result<Self> {
        Ok(Self { handle })
    }

    pub fn is_enabled(&self) -> Result<bool> {
        let control = self.handle.cast::<MUXC::Control>()?;
        control.IsEnabled()
    }

    pub fn set_enabled(&self, enabled: bool) -> Result<()> {
        let control = self.handle.cast::<MUXC::Control>()?;
        control.SetIsEnabled(enabled)?;
        Ok(())
    }
}

impl AsToolBarItem for ToolBarItem {
    fn as_tool_bar_item(&self) -> BorrowedToolBarItem<'_> {
        BorrowedToolBarItem::winui(&self.handle)
    }
}
