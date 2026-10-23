use std::rc::Rc;

use winio_handle::AsWindow;

use crate::{
    Error, Result,
    menu::system::{self, MenuBarData},
};

#[derive(Debug)]
pub struct MenuBar {
    data: Rc<MenuBarData>,
}

impl MenuBar {
    pub fn new(_parent: impl AsWindow) -> Result<Self> {
        let data = MenuBarData::new();
        if !system::install(&data) {
            return Err(Error::MenuBarExists);
        }
        Ok(Self { data })
    }

    pub fn is_visible(&self) -> Result<bool> {
        Ok(self.data.is_visible())
    }

    pub fn set_visible(&mut self, visible: bool) -> Result<()> {
        self.data.set_visible(visible);
        Ok(())
    }

    pub(crate) fn data(&self) -> &Rc<MenuBarData> {
        &self.data
    }
}

impl Drop for MenuBar {
    fn drop(&mut self) {
        system::uninstall(&self.data);
    }
}
