use inherit_methods_macro::inherit_methods;

use super::{ItemKind, ToolBarItem, bar::ToolBar};
use crate::{Image, Result};

#[derive(Debug)]
pub struct ToolBarCheckBox {
    item: ToolBarItem,
}

#[inherit_methods(from = "self.item")]
impl ToolBarCheckBox {
    pub fn new(parent: &ToolBar) -> Result<Self> {
        Ok(Self {
            item: ToolBarItem::new(parent, ItemKind::CheckBox)?,
        })
    }

    pub fn is_enabled(&self) -> Result<bool>;

    pub fn set_enabled(&self, enabled: bool) -> Result<()>;

    pub fn text(&self) -> Result<String>;

    pub fn set_text(&self, text: impl AsRef<str>) -> Result<()>;

    pub fn is_checked(&self) -> Result<bool>;

    pub fn set_checked(&self, checked: bool) -> Result<()>;

    pub fn set_icon(&self, icon: Option<&Image>) -> Result<()>;

    pub async fn wait_click(&self) {
        self.item.wait_click().await;
    }
}

winio_handle::impl_as_tool_bar_item!(ToolBarCheckBox, item);
