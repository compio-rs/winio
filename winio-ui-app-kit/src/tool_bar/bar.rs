use std::{
    cell::RefCell,
    collections::HashMap,
    rc::Rc,
    sync::atomic::{AtomicUsize, Ordering},
};

use objc2::{
    DeclaredClass, MainThreadMarker, MainThreadOnly, define_class, msg_send,
    rc::{Allocated, Retained, Weak},
    runtime::ProtocolObject,
};
use objc2_app_kit::{
    NSToolbar, NSToolbarDelegate, NSToolbarDisplayMode, NSToolbarItem, NSToolbarItemIdentifier,
    NSWindow,
};
use objc2_foundation::{NSArray, NSObject, NSObjectProtocol, NSString};
use winio_handle::AsWindow;

use crate::{Error, Result, catch, from_nsstring};

static TOOLBAR_ID: AtomicUsize = AtomicUsize::new(0);

#[derive(Debug, Default)]
struct ToolBarDelegateIvars {
    items: RefCell<HashMap<String, Retained<NSToolbarItem>>>,
}

define_class! {
    #[unsafe(super(NSObject))]
    #[name = "WinioToolBarDelegate"]
    #[ivars = ToolBarDelegateIvars]
    #[thread_kind = MainThreadOnly]
    #[derive(Debug)]
    struct ToolBarDelegate;

    #[allow(non_snake_case)]
    impl ToolBarDelegate {
        #[unsafe(method_id(init))]
        fn init(this: Allocated<Self>) -> Option<Retained<Self>> {
            let this = this.set_ivars(ToolBarDelegateIvars::default());
            unsafe { msg_send![super(this), init] }
        }
    }

    unsafe impl NSObjectProtocol for ToolBarDelegate {}

    #[allow(non_snake_case)]
    unsafe impl NSToolbarDelegate for ToolBarDelegate {
        #[unsafe(method_id(toolbar:itemForItemIdentifier:willBeInsertedIntoToolbar:))]
        unsafe fn toolbar_itemForItemIdentifier_willBeInsertedIntoToolbar(
            &self,
            _toolbar: &NSToolbar,
            item_identifier: &NSToolbarItemIdentifier,
            _will_be_inserted: bool,
        ) -> Option<Retained<NSToolbarItem>> {
            self.ivars()
                .items
                .borrow()
                .get(&from_nsstring(item_identifier))
                .cloned()
        }

        #[unsafe(method_id(toolbarDefaultItemIdentifiers:))]
        unsafe fn toolbarDefaultItemIdentifiers(
            &self,
            _toolbar: &NSToolbar,
        ) -> Retained<NSArray<NSToolbarItemIdentifier>> {
            NSArray::from_retained_slice(&[])
        }

        #[unsafe(method_id(toolbarAllowedItemIdentifiers:))]
        unsafe fn toolbarAllowedItemIdentifiers(
            &self,
            _toolbar: &NSToolbar,
        ) -> Retained<NSArray<NSToolbarItemIdentifier>> {
            let identifiers: Vec<Retained<NSToolbarItemIdentifier>> = self
                .ivars()
                .items
                .borrow()
                .keys()
                .map(|identifier| NSString::from_str(identifier))
                .collect();
            NSArray::from_retained_slice(&identifiers)
        }
    }
}

impl ToolBarDelegate {
    fn new(mtm: MainThreadMarker) -> Retained<Self> {
        unsafe { msg_send![mtm.alloc::<Self>(), init] }
    }

    fn register(&self, identifier: &NSString, item: Retained<NSToolbarItem>) {
        self.ivars()
            .items
            .borrow_mut()
            .insert(from_nsstring(identifier), item);
    }

    fn unregister(&self, identifier: &NSString) {
        self.ivars()
            .items
            .borrow_mut()
            .remove(&from_nsstring(identifier));
    }
}

#[derive(Debug)]
pub(crate) struct ToolBarData {
    window: Weak<NSWindow>,
    toolbar: Retained<NSToolbar>,
    delegate: Retained<ToolBarDelegate>,
}

impl ToolBarData {
    pub(crate) fn toolbar(&self) -> &Retained<NSToolbar> {
        &self.toolbar
    }

    pub(crate) fn register_item(&self, identifier: &NSString, item: Retained<NSToolbarItem>) {
        self.delegate.register(identifier, item);
    }

    pub(crate) fn unregister_item(&self, identifier: &NSString) {
        self.delegate.unregister(identifier);
    }

    pub(crate) fn mtm(&self) -> MainThreadMarker {
        self.toolbar.mtm()
    }
}

/// A toolbar docked in the title bar of a window.
#[derive(Debug)]
pub struct ToolBar {
    data: Rc<ToolBarData>,
}

impl ToolBar {
    pub fn new(parent: impl AsWindow) -> Result<Self> {
        let parent = parent.as_window();
        let window = parent.as_app_kit();
        if catch(|| window.toolbar().is_some())? {
            return Err(Error::ToolBarExists);
        }

        let mtm = window.mtm();
        catch(|| {
            let identifier = NSString::from_str(&format!(
                "winio.toolbar.{}",
                TOOLBAR_ID.fetch_add(1, Ordering::Relaxed)
            ));
            let toolbar = NSToolbar::initWithIdentifier(mtm.alloc(), &identifier);
            toolbar.setAllowsUserCustomization(false);
            toolbar.setAutosavesConfiguration(false);
            toolbar.setDisplayMode(NSToolbarDisplayMode::IconOnly);

            let delegate = ToolBarDelegate::new(mtm);
            toolbar.setDelegate(Some(ProtocolObject::from_ref(&*delegate)));

            window.setToolbar(Some(&toolbar));
            toolbar.setVisible(true);

            Ok(Self {
                data: Rc::new(ToolBarData {
                    window: Weak::from_retained(window),
                    toolbar,
                    delegate,
                }),
            })
        })
        .flatten()
    }

    pub fn is_visible(&self) -> Result<bool> {
        catch(|| {
            let Some(window) = self.data.window.load() else {
                return false;
            };
            window.toolbar().as_deref() == Some(&*self.data.toolbar)
                && self.data.toolbar.isVisible()
        })
    }

    pub fn set_visible(&mut self, visible: bool) -> Result<()> {
        catch(|| {
            let window = self.data.window.load().ok_or(Error::NullPointer)?;
            let attached = window.toolbar().as_deref() == Some(&*self.data.toolbar);
            if !attached && visible {
                window.setToolbar(Some(&self.data.toolbar));
            }
            self.data.toolbar.setVisible(visible);
            Ok(())
        })
        .flatten()
    }

    pub(crate) fn data(&self) -> &Rc<ToolBarData> {
        &self.data
    }
}

impl Drop for ToolBar {
    fn drop(&mut self) {
        let Some(window) = self.data.window.load() else {
            return;
        };
        if window.toolbar().as_deref() == Some(&*self.data.toolbar) {
            window.setToolbar(None);
        }
    }
}
