use gtk4::prelude::*;
use winio_handle::AsWindow;

use crate::{Error, Result};

const TOOLBAR_CSS_CLASS: &str = "winio-tool-bar";

#[derive(Debug)]
pub struct ToolBar {
    toolbar: gtk4::Box,
}

impl ToolBar {
    pub fn new(parent: impl AsWindow) -> Result<Self> {
        let window = parent.as_window().to_gtk();
        let content = window
            .child()
            .and_then(|child| child.downcast::<gtk4::Box>().ok())
            .ok_or(Error::CastFailed)?;

        let mut child = content.first_child();
        while let Some(c) = child {
            if c.has_css_class(TOOLBAR_CSS_CLASS) {
                return Err(Error::ToolBarExists);
            }
            child = c.next_sibling();
        }

        let toolbar = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        toolbar.add_css_class(TOOLBAR_CSS_CLASS);
        toolbar.add_css_class("toolbar");
        toolbar.set_hexpand(true);

        crate::menu::register_actions(window);

        let menu_bar = {
            let mut child = content.first_child();
            let mut menu_bar = None;
            while let Some(c) = child {
                if c.is::<gtk4::PopoverMenuBar>() {
                    menu_bar = Some(c);
                    break;
                }
                child = c.next_sibling();
            }
            menu_bar
        };

        if let Some(menu_bar) = menu_bar {
            content.insert_child_after(&toolbar, Some(&menu_bar));
        } else {
            content.prepend(&toolbar);
        }

        Ok(Self { toolbar })
    }

    pub fn is_visible(&self) -> Result<bool> {
        Ok(self.toolbar.get_visible())
    }

    pub fn set_visible(&mut self, visible: bool) -> Result<()> {
        self.toolbar.set_visible(visible);
        Ok(())
    }

    pub(crate) fn toolbar(&self) -> &gtk4::Box {
        &self.toolbar
    }
}

impl Drop for ToolBar {
    fn drop(&mut self) {
        self.toolbar.unparent();
    }
}
