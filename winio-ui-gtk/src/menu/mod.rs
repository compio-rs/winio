use std::cell::Cell;

use gtk4::gio;

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
