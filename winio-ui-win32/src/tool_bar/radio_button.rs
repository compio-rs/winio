use inherit_methods_macro::inherit_methods;

use super::{ItemKind, ToolBarItem, bar::ToolBar};
use crate::{Image, Result};

#[derive(Debug)]
pub struct ToolBarRadioButton {
    item: ToolBarItem,
}

#[inherit_methods(from = "self.item")]
impl ToolBarRadioButton {
    pub fn new(parent: &ToolBar) -> Result<Self> {
        Ok(Self {
            item: ToolBarItem::new(parent, ItemKind::RadioButton)?,
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
        let _ = self.item.set_checked(true);
    }
}

winio_handle::impl_as_tool_bar_item!(ToolBarRadioButton, item);
