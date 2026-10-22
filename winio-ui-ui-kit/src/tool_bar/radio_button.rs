use inherit_methods_macro::inherit_methods;

use crate::{
    Image, Result,
    tool_bar::{ToolBar, ToolBarButtonBase},
};

#[derive(Debug)]
pub struct ToolBarRadioButton {
    item: ToolBarButtonBase,
}

#[inherit_methods(from = "self.item")]
impl ToolBarRadioButton {
    pub fn new(parent: &ToolBar) -> Result<Self> {
        Ok(Self {
            item: ToolBarButtonBase::new(parent, true, false)?,
        })
    }

    pub fn is_enabled(&self) -> Result<bool>;

    pub fn set_enabled(&mut self, enabled: bool) -> Result<()>;

    pub fn text(&self) -> Result<String>;

    pub fn set_text(&mut self, text: impl AsRef<str>) -> Result<()>;

    pub fn is_checked(&self) -> Result<bool>;

    pub fn set_checked(&mut self, checked: bool) -> Result<()>;

    pub fn set_icon(&mut self, icon: Option<&Image>) -> Result<()>;

    pub async fn wait_click(&self) {
        self.item.wait_click().await;
    }
}

winio_handle::impl_as_tool_bar_item!(ToolBarRadioButton, item);
