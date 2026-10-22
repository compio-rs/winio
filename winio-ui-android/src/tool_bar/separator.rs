use jni::objects::JObject;
use winio_handle::{AsToolBarItem, BorrowedToolBarItem};

use crate::{Result, tool_bar::ToolBar};

#[derive(Debug)]
pub struct ToolBarSeparator {
    item: JObject<'static>,
}

impl ToolBarSeparator {
    pub fn new(_parent: &ToolBar) -> Result<Self> {
        Ok(Self {
            item: JObject::null(),
        })
    }
}

impl AsToolBarItem for ToolBarSeparator {
    fn as_tool_bar_item(&self) -> BorrowedToolBarItem<'_> {
        unsafe { BorrowedToolBarItem::android(&self.item) }
    }
}
