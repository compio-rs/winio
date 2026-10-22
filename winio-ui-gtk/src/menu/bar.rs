use gtk4::{gio, prelude::*};
use winio_handle::AsWindow;

use crate::{
    Error, Result,
    menu::{register_actions, register_menu_bar},
};

#[derive(Debug)]
pub struct MenuBar {
    widget: gtk4::PopoverMenuBar,
    menu: gio::Menu,
}

impl MenuBar {
    pub fn new(parent: impl AsWindow) -> Result<Self> {
        let window = parent.as_window().to_gtk();
        let content = window
            .child()
            .and_then(|child| child.downcast::<gtk4::Box>().ok())
            .ok_or(Error::CastFailed)?;

        let mut child = content.first_child();
        while let Some(c) = child {
            if c.is::<gtk4::PopoverMenuBar>() {
                return Err(Error::MenuBarExists);
            }
            child = c.next_sibling();
        }

        let menu = gio::Menu::new();
        register_actions(window);

        let widget = gtk4::PopoverMenuBar::from_model(Some(&menu));
        register_menu_bar(&widget);
        content.prepend(&widget);
        Ok(Self { widget, menu })
    }

    pub fn is_visible(&self) -> Result<bool> {
        Ok(self.widget.get_visible())
    }

    pub fn set_visible(&mut self, visible: bool) -> Result<()> {
        self.widget.set_visible(visible);
        Ok(())
    }

    pub(crate) fn menu(&self) -> &gio::Menu {
        &self.menu
    }
}

impl Drop for MenuBar {
    fn drop(&mut self) {
        self.widget.unparent();
    }
}
