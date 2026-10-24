use inherit_methods_macro::inherit_methods;
use winio_handle::{
    AsMenuContainer, AsToolBarItem, AsWindow, BorrowedMenuContainer, BorrowedToolBarItem,
};

use crate::{Image, Result, not_impl};

#[derive(Debug)]
pub(crate) struct ToolBarHandle;

impl ToolBarHandle {
    pub fn is_visible(&self) -> Result<bool> {
        not_impl()
    }

    pub fn set_visible(&mut self, _v: bool) -> Result<()> {
        not_impl()
    }

    pub fn is_enabled(&self) -> Result<bool> {
        not_impl()
    }

    pub fn set_enabled(&mut self, _v: bool) -> Result<()> {
        not_impl()
    }

    pub fn text(&self) -> Result<String> {
        not_impl()
    }

    pub fn set_text(&mut self, _s: impl AsRef<str>) -> Result<()> {
        not_impl()
    }

    pub fn set_icon(&mut self, _icon: Option<&Image>) -> Result<()> {
        not_impl()
    }

    pub fn is_checked(&self) -> Result<bool> {
        not_impl()
    }

    pub fn set_checked(&mut self, _v: bool) -> Result<()> {
        not_impl()
    }
}

impl AsToolBarItem for ToolBarHandle {
    fn as_tool_bar_item(&self) -> BorrowedToolBarItem<'_> {
        not_impl()
    }
}

impl AsMenuContainer for ToolBarHandle {
    fn as_menu_container(&self) -> BorrowedMenuContainer<'_> {
        not_impl()
    }
}

#[derive(Debug)]
pub struct ToolBar {
    handle: ToolBarHandle,
}

#[inherit_methods(from = "self.handle")]
impl ToolBar {
    pub fn new(_parent: impl AsWindow) -> Result<Self> {
        not_impl()
    }

    pub fn is_visible(&self) -> Result<bool>;

    pub fn set_visible(&mut self, v: bool) -> Result<()>;
}

#[derive(Debug)]
pub struct ToolBarButton {
    handle: ToolBarHandle,
}

#[inherit_methods(from = "self.handle")]
impl ToolBarButton {
    pub fn new(_parent: &ToolBar) -> Result<Self> {
        not_impl()
    }

    pub fn is_enabled(&self) -> Result<bool>;

    pub fn set_enabled(&mut self, v: bool) -> Result<()>;

    pub fn text(&self) -> Result<String>;

    pub fn set_text(&mut self, s: impl AsRef<str>) -> Result<()>;

    pub fn set_icon(&mut self, icon: Option<&Image>) -> Result<()>;

    pub async fn wait_click(&self) {
        not_impl()
    }
}

winio_handle::impl_as_tool_bar_item!(ToolBarButton, handle);

#[derive(Debug)]
pub struct ToolBarCheckBox {
    handle: ToolBarHandle,
}

#[inherit_methods(from = "self.handle")]
impl ToolBarCheckBox {
    pub fn new(_parent: &ToolBar) -> Result<Self> {
        not_impl()
    }

    pub fn is_enabled(&self) -> Result<bool>;

    pub fn set_enabled(&mut self, v: bool) -> Result<()>;

    pub fn text(&self) -> Result<String>;

    pub fn set_text(&mut self, s: impl AsRef<str>) -> Result<()>;

    pub fn is_checked(&self) -> Result<bool>;

    pub fn set_checked(&mut self, v: bool) -> Result<()>;

    pub fn set_icon(&mut self, icon: Option<&Image>) -> Result<()>;

    pub async fn wait_click(&self) {
        not_impl()
    }
}

winio_handle::impl_as_tool_bar_item!(ToolBarCheckBox, handle);

#[derive(Debug)]
pub struct ToolBarRadioButton {
    handle: ToolBarHandle,
}

#[inherit_methods(from = "self.handle")]
impl ToolBarRadioButton {
    pub fn new(_parent: &ToolBar) -> Result<Self> {
        not_impl()
    }

    pub fn is_enabled(&self) -> Result<bool>;

    pub fn set_enabled(&mut self, v: bool) -> Result<()>;

    pub fn text(&self) -> Result<String>;

    pub fn set_text(&mut self, s: impl AsRef<str>) -> Result<()>;

    pub fn is_checked(&self) -> Result<bool>;

    pub fn set_checked(&mut self, v: bool) -> Result<()>;

    pub fn set_icon(&mut self, icon: Option<&Image>) -> Result<()>;

    pub async fn wait_click(&self) {
        not_impl()
    }
}

winio_handle::impl_as_tool_bar_item!(ToolBarRadioButton, handle);

#[derive(Debug)]
pub struct ToolBarSeparator {
    handle: ToolBarHandle,
}

impl ToolBarSeparator {
    pub fn new(_parent: &ToolBar) -> Result<Self> {
        not_impl()
    }
}

winio_handle::impl_as_tool_bar_item!(ToolBarSeparator, handle);

#[derive(Debug)]
pub struct ToolBarMenuButton {
    handle: ToolBarHandle,
}

#[inherit_methods(from = "self.handle")]
impl ToolBarMenuButton {
    pub fn new(_parent: &ToolBar) -> Result<Self> {
        not_impl()
    }

    pub fn is_enabled(&self) -> Result<bool>;

    pub fn set_enabled(&mut self, v: bool) -> Result<()>;

    pub fn text(&self) -> Result<String>;

    pub fn set_text(&mut self, s: impl AsRef<str>) -> Result<()>;

    pub fn set_icon(&mut self, icon: Option<&Image>) -> Result<()>;

    pub async fn start(&self) -> ! {
        not_impl()
    }
}

winio_handle::impl_as_menu_container!(ToolBarMenuButton, handle);
winio_handle::impl_as_tool_bar_item!(ToolBarMenuButton, handle);
