use inherit_methods_macro::inherit_methods;

use crate::{Image, Result, menu::SubMenuButton, tool_bar::ToolBar, vm_exec};

#[derive(Debug)]
pub struct ToolBarMenuButton {
    inner: SubMenuButton,
}

#[inherit_methods(from = "self.inner")]
impl ToolBarMenuButton {
    pub fn new(parent: &ToolBar) -> Result<Self> {
        vm_exec(|env| {
            Ok(Self {
                inner: SubMenuButton::new(env, parent.menu())?,
            })
        })
    }

    pub fn is_enabled(&self) -> Result<bool>;

    pub fn set_enabled(&mut self, enabled: bool) -> Result<()>;

    pub fn text(&self) -> Result<String>;

    pub fn set_text(&mut self, s: impl AsRef<str>) -> Result<()>;

    pub fn set_icon(&mut self, icon: Option<&Image>) -> Result<()>;

    pub async fn start(&self) -> ! {
        std::future::pending().await
    }
}

winio_handle::impl_as_menu_container!(ToolBarMenuButton, inner);
winio_handle::impl_as_tool_bar_item!(ToolBarMenuButton, inner);
