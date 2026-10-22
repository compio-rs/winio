use crate::{Image, Result, menu::SubMenuItem, tool_bar::ToolBar, vm_exec};

#[derive(Debug)]
pub struct ToolBarDropDownButton {
    item: SubMenuItem,
}

impl ToolBarDropDownButton {
    pub fn new(parent: &ToolBar) -> Result<Self> {
        vm_exec(|env| {
            let item = SubMenuItem::new(env, parent.menu())?;
            item.set_show_as_action(env)?;
            Ok(Self { item })
        })
    }

    pub fn is_enabled(&self) -> Result<bool> {
        vm_exec(|env| self.item.is_enabled(env))
    }

    pub fn set_enabled(&mut self, enabled: bool) -> Result<()> {
        vm_exec(|env| self.item.set_enabled(env, enabled))
    }

    pub fn text(&self) -> Result<String> {
        vm_exec(|env| self.item.text(env))
    }

    pub fn set_text(&mut self, text: impl AsRef<str>) -> Result<()> {
        vm_exec(|env| self.item.set_text(env, text))
    }

    pub fn set_icon(&mut self, icon: Option<&Image>) -> Result<()> {
        vm_exec(|env| self.item.set_icon(env, icon))
    }

    pub async fn wait_click(&self) {
        std::future::pending().await
    }
}

winio_handle::impl_as_menu_container!(ToolBarDropDownButton, item);
winio_handle::impl_as_tool_bar_item!(ToolBarDropDownButton, item);
