use winui3::Microsoft::UI::Xaml::Controls as MUXC;

use crate::{Result, tool_bar::ToolBar};

#[derive(Debug)]
pub struct ToolBarSeparator {
    #[allow(dead_code)]
    element: MUXC::AppBarSeparator,
}

impl ToolBarSeparator {
    pub fn new(parent: &ToolBar) -> Result<Self> {
        let element = MUXC::AppBarSeparator::new()?;
        parent.element().PrimaryCommands()?.Append(&element)?;
        Ok(Self { element })
    }
}
