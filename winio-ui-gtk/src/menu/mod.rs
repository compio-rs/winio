use std::cell::{Cell, RefCell};

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
    // Actions of all the menu items of this thread. The menu bar inserts the
    // group into its window so that the `app.` prefixed actions are resolvable.
    pub(crate) static ACTIONS: gio::SimpleActionGroup = gio::SimpleActionGroup::new();

    // Menu bars of this thread. GTK creates an image for every menu item icon
    // but hides it when the item also has a label, so the icon has to be made
    // visible again after the model changes.
    static MENU_BARS: RefCell<Vec<glib::WeakRef<gtk4::PopoverMenuBar>>> =
        const { RefCell::new(Vec::new()) };
}

pub(crate) fn register_menu_bar(bar: &gtk4::PopoverMenuBar) {
    MENU_BARS.with_borrow_mut(|bars| {
        bars.retain(|weak| weak.upgrade().is_some());
        let weak = glib::WeakRef::new();
        weak.set(Some(bar));
        bars.push(weak);
    });
}

pub(crate) fn refresh_menu_icons() {
    MENU_BARS.with_borrow_mut(|bars| {
        bars.retain(|weak| match weak.upgrade() {
            Some(bar) => {
                show_menu_icons(bar.upcast_ref());
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
