use inherit_methods_macro::inherit_methods;

use super::{ItemKind, ToolBarItem, bar::ToolBar};
use crate::{Image, Result};

#[derive(Debug)]
pub struct ToolBarButton {
    item: ToolBarItem,
}

#[inherit_methods(from = "self.item")]
impl ToolBarButton {
    pub fn new(parent: &ToolBar) -> Result<Self> {
        Ok(Self {
            item: ToolBarItem::new(parent, ItemKind::Button)?,
        })
    }

    pub fn is_enabled(&self) -> Result<bool>;

    pub fn set_enabled(&self, enabled: bool) -> Result<()>;

    pub fn text(&self) -> Result<String>;

    pub fn set_text(&self, text: impl AsRef<str>) -> Result<()>;

    pub fn set_icon(&self, icon: Option<&Image>) -> Result<()>;

    pub async fn wait_click(&self) {
        self.item.wait_click().await;
    }
}

winio_handle::impl_as_tool_bar_item!(ToolBarButton, item);
