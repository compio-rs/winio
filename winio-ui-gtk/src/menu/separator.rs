use gtk4::{gio, glib::WeakRef, prelude::*};
use winio_handle::{AsMenuContainer, AsMenuItem, BorrowedMenuItem};

use crate::{
    Result,
    menu::{
        SECTION_DATA, menu_item_position, menu_key, register_menu_item, unregister_menu_item,
        unregister_menu_item_by_key,
    },
};

pub struct SeparatorMenuItem {
    parent: WeakRef<gio::Menu>,
    parent_key: usize,
    section: WeakRef<gio::Menu>,
    item: gio::MenuItem,
}

impl std::fmt::Debug for SeparatorMenuItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SeparatorMenuItem")
            .field("item", &self.item)
            .finish_non_exhaustive()
    }
}

impl SeparatorMenuItem {
    pub fn new(parent: impl AsMenuContainer) -> Result<Self> {
        let parent = parent.as_menu_container().to_gtk();

        let section = gio::Menu::new();
        let item = gio::MenuItem::new_section(None, &section);
        let parent_key = menu_key(parent);
        parent.append_item(&item);
        register_menu_item(parent, &item);
        // Following items join the new section, which makes GTK draw a
        // separator before them.
        unsafe { parent.set_data(SECTION_DATA, section.clone()) };

        let parent_weak = WeakRef::new();
        parent_weak.set(Some(parent));
        let section_weak = WeakRef::new();
        section_weak.set(Some(&section));
        Ok(Self {
            parent: parent_weak,
            parent_key,
            section: section_weak,
            item,
        })
    }
}

impl Drop for SeparatorMenuItem {
    fn drop(&mut self) {
        let Some(parent) = self.parent.upgrade() else {
            unregister_menu_item_by_key(self.parent_key, &self.item);
            return;
        };
        if let Some(index) = menu_item_position(&parent, &self.item) {
            parent.remove(index as _);
        }
        unregister_menu_item(&parent, &self.item);
        unsafe {
            if let Some(section) = self.section.upgrade()
                && parent
                    .data::<gio::Menu>(SECTION_DATA)
                    .is_some_and(|current| std::ptr::eq(current.as_ref(), &section))
            {
                parent.set_data(SECTION_DATA, None::<gio::Menu>);
            }
        }
    }
}

impl AsMenuItem for SeparatorMenuItem {
    fn as_menu_item(&self) -> BorrowedMenuItem<'_> {
        BorrowedMenuItem::gtk(&self.item)
    }
}
