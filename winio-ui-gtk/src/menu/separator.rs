use gtk4::{gio, prelude::*};
use winio_handle::{AsMenuContainer, AsMenuItem, BorrowedMenuItem};

use crate::{Result, menu::SECTION_DATA};

#[derive(Debug)]
pub struct SeparatorMenuItem {
    item: gio::MenuItem,
}

impl SeparatorMenuItem {
    pub fn new(parent: impl AsMenuContainer) -> Result<Self> {
        let parent = parent.as_menu_container().to_gtk();

        let menu = gio::Menu::new();
        let item = gio::MenuItem::new_section(None, &menu);
        parent.append_item(&item);
        // Following items join the new section, which makes GTK draw a
        // separator before them.
        unsafe { parent.set_data(SECTION_DATA, menu) };
        Ok(Self { item })
    }
}

impl AsMenuItem for SeparatorMenuItem {
    fn as_menu_item(&self) -> BorrowedMenuItem<'_> {
        BorrowedMenuItem::gtk(&self.item)
    }
}
