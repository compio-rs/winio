use std::{
    cell::{Cell, RefCell},
    collections::HashMap,
};

use gtk4::{gio, glib, prelude::*};

mod bar;
pub use bar::*;

mod bar_item;
pub use bar_item::*;

mod item;
pub use item::*;

mod popup;
pub use popup::*;

mod check;
pub use check::*;

mod radio;
pub use radio::*;

mod separator;
pub use separator::*;

/// Prefix of the menu actions inserted into the window of a menu bar.
pub(crate) const ACTION_PREFIX: &str = "app";

/// Key of the section that following items are appended to.
pub(crate) const SECTION_DATA: &str = "winio-menu-section";

thread_local! {
    // Actions of all the menu items of this thread. The menu bar or tool bar
    // inserts the group into its window so that the `app.` prefixed actions are
    // resolvable.
    pub(crate) static ACTIONS: gio::SimpleActionGroup = gio::SimpleActionGroup::new();

    // Menu bars and popovers of this thread. GTK creates an image for every
    // menu item icon but hides it when the item also has a label, so the icon
    // has to be made visible again after the model changes and whenever a
    // popover is opened.
    static MENUS: RefCell<Vec<glib::WeakRef<gtk4::Widget>>> =
        const { RefCell::new(Vec::new()) };

    // The items of every menu model, in model order. `gio::Menu` has no way to
    // look up the position of a `gio::MenuItem`, so keep track of it here to
    // support removing an item when it is dropped.
    static MENU_ITEMS: RefCell<HashMap<usize, Vec<usize>>> = RefCell::new(HashMap::new());
}

pub(crate) fn menu_key(menu: &gio::Menu) -> usize {
    menu.as_ptr() as usize
}

fn menu_item_key(item: &gio::MenuItem) -> usize {
    item.as_ptr() as usize
}

pub(crate) fn register_menu_item(parent: &gio::Menu, item: &gio::MenuItem) {
    MENU_ITEMS.with_borrow_mut(|items| {
        items
            .entry(menu_key(parent))
            .or_default()
            .push(menu_item_key(item));
    });
}

pub(crate) fn menu_item_position(parent: &gio::Menu, item: &gio::MenuItem) -> Option<usize> {
    MENU_ITEMS.with_borrow(|items| {
        items
            .get(&menu_key(parent))?
            .iter()
            .position(|value| *value == menu_item_key(item))
    })
}

pub(crate) fn unregister_menu_item(parent: &gio::Menu, item: &gio::MenuItem) {
    unregister_menu_item_by_key(menu_key(parent), item);
}

pub(crate) fn unregister_menu_item_by_key(key: usize, item: &gio::MenuItem) {
    MENU_ITEMS.with_borrow_mut(|items| {
        let empty = if let Some(entries) = items.get_mut(&key) {
            if let Some(index) = entries
                .iter()
                .position(|value| *value == menu_item_key(item))
            {
                entries.remove(index);
            }
            entries.is_empty()
        } else {
            false
        };
        if empty {
            items.remove(&key);
        }
    });
}

pub(crate) fn register_actions(window: &gtk4::Window) {
    ACTIONS.with(|actions| window.insert_action_group(ACTION_PREFIX, Some(actions)));
}

pub(crate) fn register_menu_bar(bar: &gtk4::PopoverMenuBar) {
    register_menu(bar.upcast_ref());
}

pub(crate) fn register_menu_popover(popover: &gtk4::PopoverMenu) {
    if register_menu(popover.upcast_ref()) {
        let weak = glib::WeakRef::new();
        weak.set(Some(popover));
        popover.connect_show(move |_| {
            if let Some(popover) = weak.upgrade() {
                glib::idle_add_local_once(move || show_menu_icons(popover.upcast_ref()));
            }
        });
    }
}

fn register_menu(menu: &gtk4::Widget) -> bool {
    let added = MENUS.with_borrow_mut(|menus| {
        menus.retain(|weak| weak.upgrade().is_some());
        if menus
            .iter()
            .filter_map(|weak| weak.upgrade())
            .any(|w| w == *menu)
        {
            return false;
        }
        let weak = glib::WeakRef::new();
        weak.set(Some(menu));
        menus.push(weak);
        true
    });
    if added {
        show_menu_icons(menu);
    }
    added
}

pub(crate) fn refresh_menu_icons() {
    MENUS.with_borrow_mut(|menus| {
        menus.retain(|weak| match weak.upgrade() {
            Some(menu) => {
                show_menu_icons(&menu);
                true
            }
            None => false,
        });
    });
}

fn show_menu_icons(widget: &gtk4::Widget) {
    if widget.type_().name() == "GtkModelButton" {
        let mut start_box = None;
        let mut icon = None;
        let mut child = widget.first_child();
        while let Some(c) = child {
            if let Some(b) = c.downcast_ref::<gtk4::Box>() {
                if start_box.is_none() {
                    start_box = Some(b.clone());
                }
            } else if let Some(image) = c.downcast_ref::<gtk4::Image>() {
                icon = Some(image.clone());
            }
            child = c.next_sibling();
        }

        if let Some(start_box) = start_box {
            if let Some(icon) = icon {
                if start_box.first_child().is_none() {
                    icon.unparent();
                    start_box.append(&icon);
                }
                show_icon(&icon);
            } else {
                let mut child = start_box.first_child();
                while let Some(c) = child {
                    if let Some(icon) = c.downcast_ref::<gtk4::Image>() {
                        show_icon(icon);
                    }
                    child = c.next_sibling();
                }
            }
        } else if let Some(icon) = icon {
            show_icon(&icon);
        }
    }

    let mut child = widget.first_child();
    while let Some(c) = child {
        show_menu_icons(&c);
        child = c.next_sibling();
    }
}

fn show_icon(icon: &gtk4::Image) {
    icon.set_halign(gtk4::Align::Center);
    icon.set_valign(gtk4::Align::Center);
    icon.set_visible(true);
}

fn alloc_action_name() -> String {
    thread_local! {
        static COUNTER: Cell<usize> = const { Cell::new(0) };
    }
    let i = COUNTER.with(|c| {
        let i = c.get();
        c.set(i + 1);
        i
    });
    format!("winiomenu{}", i)
}
