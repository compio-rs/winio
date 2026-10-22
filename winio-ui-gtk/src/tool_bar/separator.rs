use gtk4::prelude::*;
use winio_handle::{AsToolBarItem, BorrowedToolBarItem};

use crate::{Result, tool_bar::ToolBar};

#[derive(Debug)]
pub struct ToolBarSeparator {
    separator: gtk4::Separator,
}

impl ToolBarSeparator {
    pub fn new(parent: &ToolBar) -> Result<Self> {
        let separator = gtk4::Separator::new(gtk4::Orientation::Vertical);
        separator.set_margin_start(4);
        separator.set_margin_end(4);
        parent.toolbar().append(&separator);
        Ok(Self { separator })
    }
}

impl AsToolBarItem for ToolBarSeparator {
    fn as_tool_bar_item(&self) -> BorrowedToolBarItem<'_> {
        BorrowedToolBarItem::gtk(self.separator.upcast_ref::<gtk4::Widget>())
    }
}
