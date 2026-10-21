use super::{ItemKind, ToolBarItem, bar::ToolBar};
use crate::Result;

#[derive(Debug)]
pub struct ToolBarSeparator {
    item: ToolBarItem,
}

impl ToolBarSeparator {
    pub fn new(parent: &ToolBar) -> Result<Self> {
        Ok(Self {
            item: ToolBarItem::new(parent, ItemKind::Separator)?,
        })
    }
}

winio_handle::impl_as_tool_bar_item!(ToolBarSeparator, item);
