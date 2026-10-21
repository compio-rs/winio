use windows_core::Interface;
use winui3::Microsoft::UI::Xaml::Controls as MUXC;

use crate::{
    Result,
    tool_bar::{ToolBar, ToolBarItem},
};

#[derive(Debug)]
pub struct ToolBarSeparator {
    item: ToolBarItem,
}

impl ToolBarSeparator {
    pub fn new(parent: &ToolBar) -> Result<Self> {
        let element = MUXC::AppBarSeparator::new()?;
        parent.element().PrimaryCommands()?.Append(&element)?;
        let item = ToolBarItem::new(element.cast()?)?;
        Ok(Self { item })
    }
}

winio_handle::impl_as_tool_bar_item!(ToolBarSeparator, item);
