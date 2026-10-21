use windows_core::Interface;
use winio_handle::{AsToolBarItem, BorrowedToolBarItem};
use winui3::Microsoft::UI::Xaml::Controls as MUXC;

use crate::{Result, tool_bar::ToolBar};

#[derive(Debug)]
pub struct ToolBarSeparator {
    handle: MUXC::ICommandBarElement,
}

impl ToolBarSeparator {
    pub fn new(parent: &ToolBar) -> Result<Self> {
        let element = MUXC::AppBarSeparator::new()?;
        parent.element().PrimaryCommands()?.Append(&element)?;
        let handle = element.cast()?;
        Ok(Self { handle })
    }
}

impl AsToolBarItem for ToolBarSeparator {
    fn as_tool_bar_item(&self) -> BorrowedToolBarItem<'_> {
        BorrowedToolBarItem::winui(&self.handle)
    }
}
