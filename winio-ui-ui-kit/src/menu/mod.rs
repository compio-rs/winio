//! The menu implementation of the UIKit backend.
//!
//! UIKit menus are immutable once they are built, and they cannot be handed to
//! another menu afterwards. winio therefore keeps the menu tree on the Rust
//! side: every node lives in a slab, the widgets only hold the key of their
//! node, and the `UIMenuElement`s are rebuilt on demand by a
//! `UIDeferredMenuElement` each time a menu is about to be displayed.

use std::{cell::RefCell, ptr::NonNull, rc::Rc};

use block2::{DynBlock, RcBlock};
use objc2::{MainThreadMarker, available, msg_send, rc::Retained};
use objc2_foundation::{NSArray, NSInteger, NSString, ns_string};
use objc2_ui_kit::{
    UIAction, UIDeferredMenuElement, UIFont, UIImage, UIMenu, UIMenuElement,
    UIMenuElementAttributes, UIMenuElementState, UIMenuOptions,
};
use slab::Slab;
use winio_callback::Callback;
use winio_handle::{AsMenuContainer, AsMenuItem, BorrowedMenuContainer, BorrowedMenuItem};

use crate::{GlobalRuntime, Image, Result, catch};

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

thread_local! {
    static NODES: RefCell<Slab<MenuNode>> = const { RefCell::new(Slab::new()) };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum NodeKind {
    Root,
    Item,
    Check,
    Radio,
    Popup,
    Separator,
}

#[derive(Debug)]
struct MenuNode {
    kind: NodeKind,
    text: String,
    icon: Option<Retained<UIImage>>,
    enabled: bool,
    checked: bool,
    click: Rc<Callback>,
    parent: Option<usize>,
    children: Vec<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct MenuNodeRef(usize);

impl MenuNodeRef {
    pub fn root() -> Self {
        Self(insert_node(NodeKind::Root, None))
    }

    pub fn child(parent: usize, kind: NodeKind) -> Self {
        Self(insert_node(kind, Some(parent)))
    }

    pub fn key(&self) -> usize {
        self.0
    }

    fn with<R>(&self, f: impl FnOnce(&MenuNode) -> R) -> Option<R> {
        NODES.with_borrow(|nodes| nodes.get(self.0).map(f))
    }

    fn with_mut<R>(&self, f: impl FnOnce(&mut MenuNode) -> R) -> Option<R> {
        NODES.with_borrow_mut(|nodes| nodes.get_mut(self.0).map(f))
    }

    pub fn text(&self) -> String {
        self.with(|node| node.text.clone()).unwrap_or_default()
    }

    pub fn set_text(&self, s: impl AsRef<str>) {
        self.with_mut(|node| node.text = s.as_ref().to_string());
    }

    pub fn set_icon(&self, icon: Option<&Image>) -> Result<()> {
        catch(|| {
            let icon = match icon {
                Some(icon) => Some(icon.uiimage(Some(UIFont::systemFontSize()))?),
                None => None,
            };
            self.with_mut(|node| node.icon = icon);
            Ok(())
        })
        .flatten()
    }

    pub fn is_enabled(&self) -> bool {
        self.with(|node| node.enabled).unwrap_or(true)
    }

    pub fn set_enabled(&self, enabled: bool) {
        self.with_mut(|node| node.enabled = enabled);
    }

    pub fn is_checked(&self) -> bool {
        self.with(|node| node.checked).unwrap_or(false)
    }

    pub fn set_checked(&self, checked: bool) {
        self.with_mut(|node| node.checked = checked);
    }

    pub fn toggle_checked(&self) {
        self.with_mut(|node| node.checked = !node.checked);
    }

    pub async fn wait_click(&self) {
        let click = match self.with(|node| node.click.clone()) {
            Some(click) => click,
            // The node is gone, so it can never be clicked again.
            None => std::future::pending().await,
        };
        click.wait().await;
    }
}

impl Drop for MenuNodeRef {
    fn drop(&mut self) {
        remove_node(self.0);
    }
}

impl AsMenuContainer for MenuNodeRef {
    fn as_menu_container(&self) -> BorrowedMenuContainer<'_> {
        BorrowedMenuContainer::ui_kit(self.0)
    }
}

impl AsMenuItem for MenuNodeRef {
    fn as_menu_item(&self) -> BorrowedMenuItem<'_> {
        BorrowedMenuItem::ui_kit(self.0)
    }
}

fn insert_node(kind: NodeKind, parent: Option<usize>) -> usize {
    NODES.with_borrow_mut(|nodes| {
        let key = nodes.insert(MenuNode {
            kind,
            text: String::new(),
            icon: None,
            enabled: true,
            checked: false,
            click: Rc::new(Callback::new()),
            parent,
            children: Vec::new(),
        });
        if let Some(parent) = parent
            && let Some(node) = nodes.get_mut(parent)
        {
            node.children.push(key);
        }
        key
    })
}

fn remove_node(key: usize) {
    NODES.with_borrow_mut(|nodes| {
        let Some(node) = nodes.try_remove(key) else {
            return;
        };
        if let Some(parent) = node.parent
            && let Some(parent) = nodes.get_mut(parent)
        {
            parent.children.retain(|child| *child != key);
        }
    });
}

fn signal_click(key: usize) {
    // Clone the callback out first: signaling runs the runtime, which must not
    // happen under the borrow of the slab.
    let click = NODES.with_borrow(|nodes| nodes.get(key).map(|node| node.click.clone()));
    if let Some(click) = click {
        click.signal::<GlobalRuntime>(());
    }
}

fn node_kind(key: usize) -> Option<NodeKind> {
    NODES.with_borrow(|nodes| nodes.get(key).map(|node| node.kind))
}

fn show_menu_element_image(element: &UIMenuElement) {
    #[allow(non_upper_case_globals)]
    const UIMenuElementImageVisibilityVisible: NSInteger = 1;

    if available!(ios = 27.0) {
        unsafe {
            let () = msg_send![element, setPreferredImageVisibility: UIMenuElementImageVisibilityVisible];
        }
    }
}

pub(crate) fn build_children(key: usize, mtm: MainThreadMarker) -> Vec<Retained<UIMenuElement>> {
    let children = NODES.with_borrow(|nodes| {
        nodes
            .get(key)
            .map(|node| node.children.clone())
            .unwrap_or_default()
    });

    let mut groups: Vec<Vec<Retained<UIMenuElement>>> = vec![Vec::new()];
    for child in children {
        match node_kind(child) {
            Some(NodeKind::Separator) => groups.push(Vec::new()),
            Some(_) => {
                if let Some(element) = build_element(child, mtm) {
                    groups.last_mut().unwrap().push(element);
                }
            }
            None => {}
        }
    }
    groups.retain(|group| !group.is_empty());
    if groups.len() <= 1 {
        return groups.into_iter().next().unwrap_or_default();
    }
    groups
        .into_iter()
        .map(|group| {
            let inline = UIMenu::menuWithTitle_image_identifier_options_children(
                ns_string!(""),
                None,
                None,
                UIMenuOptions::DisplayInline,
                &NSArray::from_retained_slice(&group),
                mtm,
            );
            // SAFETY: `UIMenu` is a subclass of `UIMenuElement`.
            unsafe { Retained::cast_unchecked::<UIMenuElement>(inline) }
        })
        .collect()
}

fn build_element(key: usize, mtm: MainThreadMarker) -> Option<Retained<UIMenuElement>> {
    let (kind, text, icon, enabled, checked) = NODES.with_borrow(|nodes| {
        let node = nodes.get(key)?;
        Some((
            node.kind,
            node.text.clone(),
            node.icon.clone(),
            node.enabled,
            node.checked,
        ))
    })?;

    match kind {
        NodeKind::Popup => {
            let submenu = UIMenu::menuWithTitle_image_identifier_options_children(
                &NSString::from_str(&text),
                icon.as_deref(),
                None,
                UIMenuOptions::empty(),
                &NSArray::from_retained_slice(&build_children(key, mtm)),
                mtm,
            );
            if icon.is_some() {
                show_menu_element_image(&submenu);
            }
            // SAFETY: `UIMenu` is a subclass of `UIMenuElement`.
            Some(unsafe { Retained::cast_unchecked::<UIMenuElement>(submenu) })
        }
        NodeKind::Item | NodeKind::Check | NodeKind::Radio => {
            let handler = RcBlock::new(move |_: NonNull<UIAction>| signal_click(key));
            // SAFETY: `UIAction` copies the block.
            let action = unsafe { UIAction::actionWithHandler(RcBlock::as_ptr(&handler), mtm) };
            action.setTitle(&NSString::from_str(&text));
            action.setImage(icon.as_deref());
            if icon.is_some() {
                show_menu_element_image(&action);
            }
            action.setAttributes(if enabled {
                UIMenuElementAttributes::empty()
            } else {
                UIMenuElementAttributes::Disabled
            });
            action.setState(if checked {
                UIMenuElementState::On
            } else {
                UIMenuElementState::Off
            });
            // SAFETY: `UIAction` is a subclass of `UIMenuElement`.
            Some(unsafe { Retained::cast_unchecked::<UIMenuElement>(action) })
        }
        NodeKind::Root | NodeKind::Separator => None,
    }
}

pub(crate) fn build_bar_menu(key: usize, mtm: MainThreadMarker) -> Retained<UIMenu> {
    type Completion = DynBlock<dyn Fn(NonNull<NSArray<UIMenuElement>>)>;

    let provider = RcBlock::new(move |completion: NonNull<Completion>| {
        let elements = build_children(key, mtm);
        let array = NSArray::from_retained_slice(&elements);
        // SAFETY: the system passes a valid completion block, which may be
        // called synchronously.
        unsafe { completion.as_ref() }.call((NonNull::from(&*array),));
    });
    // SAFETY: `UIDeferredMenuElement` copies the block.
    let deferred = unsafe { UIDeferredMenuElement::elementWithUncachedProvider(&provider, mtm) };
    // SAFETY: `UIDeferredMenuElement` is a subclass of `UIMenuElement`.
    let deferred = unsafe { Retained::cast_unchecked::<UIMenuElement>(deferred) };
    UIMenu::menuWithChildren(&NSArray::from_retained_slice(&[deferred]), mtm)
}
