use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

use dispatch2::DispatchQueue;
use objc2::{MainThreadMarker, MainThreadOnly, runtime::ProtocolObject};
use objc2_foundation::{NSArray, NSString};
use objc2_ui_kit::{
    UIMenu, UIMenuBuilder, UIMenuEdit, UIMenuFile, UIMenuFormat, UIMenuHelp, UIMenuIdentifier,
    UIMenuOptions, UIMenuSystem, UIMenuView, UIMenuWindow,
};

use super::NODES;

thread_local! {
    static OWNER: RefCell<Option<Rc<MenuBarData>>> = const { RefCell::new(None) };
    static REBUILD_PENDING: Cell<bool> = const { Cell::new(false) };
}

#[derive(Debug)]
pub(crate) struct MenuBarData {
    items: RefCell<Vec<usize>>,
    visible: Cell<bool>,
}

impl MenuBarData {
    pub(crate) fn new() -> Rc<Self> {
        Rc::new(Self {
            items: RefCell::new(Vec::new()),
            visible: Cell::new(true),
        })
    }

    pub(crate) fn add_item(&self, key: usize) {
        self.items.borrow_mut().push(key);
        rebuild();
    }

    pub(crate) fn remove_item(&self, key: usize) {
        self.items.borrow_mut().retain(|item| *item != key);
        rebuild();
    }

    pub(crate) fn is_visible(&self) -> bool {
        self.visible.get() && is_owner(self)
    }

    pub(crate) fn set_visible(&self, visible: bool) {
        if self.visible.get() != visible {
            self.visible.set(visible);
            rebuild();
        }
    }
}

pub(crate) fn install(data: &Rc<MenuBarData>) -> bool {
    let installed = OWNER.with_borrow_mut(|owner| {
        if owner.is_some() {
            false
        } else {
            *owner = Some(data.clone());
            true
        }
    });
    if !installed {
        return false;
    }
    rebuild();
    true
}

pub(crate) fn uninstall(data: &Rc<MenuBarData>) {
    let removed = OWNER.with_borrow_mut(|owner| {
        let is_this = owner.as_ref().is_some_and(|owner| Rc::ptr_eq(owner, data));
        if is_this {
            *owner = None;
        }
        is_this
    });
    if removed {
        rebuild();
    }
}

fn is_owner(data: &MenuBarData) -> bool {
    OWNER.with_borrow(|owner| {
        owner
            .as_deref()
            .is_some_and(|owner| std::ptr::eq(owner, data))
    })
}

pub(crate) fn rebuild() {
    REBUILD_PENDING.with(|pending| {
        if pending.get() {
            return;
        }
        pending.set(true);
        DispatchQueue::main().exec_async(|| {
            REBUILD_PENDING.with(|pending| pending.set(false));
            let mtm = MainThreadMarker::new().unwrap();
            UIMenuSystem::mainSystem(mtm).setNeedsRebuild();
        });
    });
}

pub(crate) fn build(builder: &ProtocolObject<dyn UIMenuBuilder>) {
    let Some(data) = OWNER.with_borrow(|owner| owner.clone()) else {
        return;
    };
    if !data.visible.get() {
        return;
    }
    let mtm = builder.mtm();

    let items = data.items.borrow().clone();
    let mut custom = Vec::new();
    for key in items {
        let Some((text, enabled)) =
            NODES.with_borrow(|nodes| nodes.get(key).map(|node| (node.text.clone(), node.enabled)))
        else {
            continue;
        };
        if text.is_empty() {
            continue;
        }

        let elements = if enabled {
            super::build_children(key, mtm)
        } else {
            super::build_children_disabled(key, mtm)
        };

        if let Some(identifier) = system_menu(&text) {
            let Some(menu) = builder.menuForIdentifier(identifier) else {
                continue;
            };
            let mut children = elements;
            children.extend(menu.children().to_vec());
            let menu = menu.menuByReplacingChildren(&NSArray::from_retained_slice(&children));
            builder.replaceMenuForIdentifier_withMenu(identifier, &menu);
        } else {
            let identifier = NSString::from_str(&format!("winio.menu.{key}"));
            let menu = UIMenu::menuWithTitle_image_identifier_options_children(
                &NSString::from_str(&text),
                None,
                Some(&identifier),
                UIMenuOptions::empty(),
                &NSArray::from_retained_slice(&elements),
                mtm,
            );
            custom.push(menu);
        }
    }

    let view = unsafe { UIMenuView };
    let window = unsafe { UIMenuWindow };
    if builder.menuForIdentifier(view).is_some() {
        for menu in custom.into_iter().rev() {
            builder.insertSiblingMenu_afterMenuForIdentifier(&menu, view);
        }
    } else if builder.menuForIdentifier(window).is_some() {
        for menu in custom {
            builder.insertSiblingMenu_beforeMenuForIdentifier(&menu, window);
        }
    }
}

fn system_menu(text: &str) -> Option<&'static UIMenuIdentifier> {
    if text.eq_ignore_ascii_case("file") {
        Some(unsafe { UIMenuFile })
    } else if text.eq_ignore_ascii_case("edit") {
        Some(unsafe { UIMenuEdit })
    } else if text.eq_ignore_ascii_case("format") {
        Some(unsafe { UIMenuFormat })
    } else if text.eq_ignore_ascii_case("view") {
        Some(unsafe { UIMenuView })
    } else if text.eq_ignore_ascii_case("window") {
        Some(unsafe { UIMenuWindow })
    } else if text.eq_ignore_ascii_case("help") {
        Some(unsafe { UIMenuHelp })
    } else {
        None
    }
}
