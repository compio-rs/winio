use std::rc::Rc;

use gtk4::{
    gio::{self, prelude::*},
    glib::{SignalHandlerId, Variant, WeakRef},
};
use inherit_methods_macro::inherit_methods;
use winio_callback::Callback;
use winio_handle::{AsMenuContainer, AsMenuItem, BorrowedMenuItem};

use crate::{
    GlobalRuntime, Image, Result,
    menu::{
        ACTION_PREFIX, ACTIONS, SECTION_DATA, alloc_action_name, menu_item_position, menu_key,
        refresh_menu_icons, register_menu_item, unregister_menu_item, unregister_menu_item_by_key,
    },
};

pub(crate) struct ItemBase {
    label: String,
    parent: WeakRef<gio::Menu>,
    parent_key: usize,
    item: gio::MenuItem,
}

impl std::fmt::Debug for ItemBase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ItemBase")
            .field("label", &self.label)
            .field("item", &self.item)
            .finish_non_exhaustive()
    }
}

impl ItemBase {
    pub fn new(
        parent: &gio::Menu,
        action: Option<&str>,
        target: Option<&Variant>,
        submenu: Option<&gio::Menu>,
    ) -> Result<Self> {
        let item = gio::MenuItem::new(Some(""), None);
        if let Some(action) = action {
            item.set_action_and_target_value(Some(action), target);
        }
        if let Some(submenu) = submenu {
            item.set_submenu(Some(submenu));
        }
        // Following items join the section started by the last separator.
        let parent = match unsafe { parent.data::<gio::Menu>(SECTION_DATA) } {
            Some(section) => unsafe { section.as_ref() }.clone(),
            None => parent.clone(),
        };
        let parent_key = menu_key(&parent);
        parent.append_item(&item);
        register_menu_item(&parent, &item);
        refresh_menu_icons();

        let parent_weak = WeakRef::new();
        parent_weak.set(Some(&parent));
        Ok(Self {
            label: String::new(),
            parent: parent_weak,
            parent_key,
            item,
        })
    }

    pub fn text(&self) -> Result<String> {
        Ok(self.label.clone())
    }

    pub fn set_text(&mut self, s: impl AsRef<str>) -> Result<()> {
        let s = s.as_ref();
        self.label = s.to_string();
        self.item.set_label(Some(s));
        self.refresh();
        Ok(())
    }

    pub fn set_icon(&mut self, icon: Option<&Image>) -> Result<()> {
        match icon {
            Some(icon) => self.item.set_icon(icon.texture()),
            None => self.item.set_attribute_value("icon", None),
        }
        self.refresh();
        Ok(())
    }

    fn refresh(&self) {
        let Some(parent) = self.parent.upgrade() else {
            return;
        };
        if let Some(index) = menu_item_position(&parent, &self.item) {
            parent.remove(index as _);
            parent.insert_item(index as _, &self.item);
            refresh_menu_icons();
        }
    }
}

impl Drop for ItemBase {
    fn drop(&mut self) {
        if let Some(parent) = self.parent.upgrade() {
            if let Some(index) = menu_item_position(&parent, &self.item) {
                parent.remove(index as _);
            }
            unregister_menu_item(&parent, &self.item);
        } else {
            unregister_menu_item_by_key(self.parent_key, &self.item);
        }
    }
}

impl AsMenuItem for ItemBase {
    fn as_menu_item(&self) -> BorrowedMenuItem<'_> {
        BorrowedMenuItem::gtk(&self.item)
    }
}

#[derive(Debug)]
pub struct MenuItem {
    item: ItemBase,
    action: gio::SimpleAction,
    on_click: Rc<Callback<()>>,
    handler: Option<SignalHandlerId>,
}

#[inherit_methods(from = "self.item")]
impl MenuItem {
    pub fn new(parent: impl AsMenuContainer) -> Result<Self> {
        let parent = parent.as_menu_container().to_gtk();

        let action_name = alloc_action_name();
        let action = gio::SimpleAction::new(&action_name, None);
        ACTIONS.with(|actions| actions.add_action(&action));
        let detailed_action = format!("{}.{}", ACTION_PREFIX, action_name);
        let item = ItemBase::new(parent, Some(&detailed_action), None, None)?;

        let on_click = Rc::new(Callback::new());
        let handler = action.connect_activate({
            let on_click = on_click.clone();
            move |_, _| on_click.signal::<GlobalRuntime>(())
        });

        Ok(Self {
            item,
            action,
            on_click,
            handler: Some(handler),
        })
    }

    pub fn is_enabled(&self) -> Result<bool> {
        Ok(self.action.is_enabled())
    }

    pub fn set_enabled(&mut self, enabled: bool) -> Result<()> {
        self.action.set_enabled(enabled);
        Ok(())
    }

    pub fn text(&self) -> Result<String>;

    pub fn set_text(&mut self, s: impl AsRef<str>) -> Result<()>;

    pub fn set_icon(&mut self, icon: Option<&Image>) -> Result<()>;

    pub async fn wait_click(&self) {
        self.on_click.wait().await
    }
}

impl Drop for MenuItem {
    fn drop(&mut self) {
        if let Some(handler) = self.handler.take() {
            self.action.disconnect(handler);
        }
    }
}

winio_handle::impl_as_menu_item!(MenuItem, item);
