use std::{
    rc::Rc,
    sync::atomic::{AtomicUsize, Ordering},
};

use objc2::{
    DeclaredClass, MainThreadMarker, MainThreadOnly, available, define_class, msg_send,
    rc::{Allocated, Retained},
    sel,
};
use objc2_app_kit::{
    NSBezelStyle, NSButton, NSButtonType, NSCellImagePosition, NSControlStateValueOff,
    NSControlStateValueOn, NSImageScaling, NSToolbarItem, NSView,
};
use objc2_foundation::{NSInteger, NSObject, NSString, ns_string};
use winio_callback::Callback;
use winio_handle::{AsToolBarItem, BorrowedToolBarItem};

use crate::{
    GlobalRuntime, Image, Result, catch, from_nsstring,
    tool_bar::{ToolBar, ToolBarData},
};

static ITEM_ID: AtomicUsize = AtomicUsize::new(0);

#[allow(deprecated)]
pub(crate) fn toolbar_bezel_style() -> NSBezelStyle {
    if available!(macos = 11.0) {
        NSBezelStyle::Toolbar
    } else {
        NSBezelStyle::TexturedRounded
    }
}

pub(crate) fn refresh_image_position(button: &NSButton) {
    let has_image = button.image().is_some();
    let has_text = !from_nsstring(&button.title()).is_empty();
    let position = match (has_image, has_text) {
        (true, true) => NSCellImagePosition::ImageLeft,
        (true, false) => NSCellImagePosition::ImageOnly,
        (false, _) => NSCellImagePosition::NoImage,
    };
    button.setImagePosition(position);
}

fn set_item_label(item: &NSToolbarItem, label: &str) {
    let label = NSString::from_str(label);
    item.setLabel(&label);
    item.setPaletteLabel(&label);
}

#[derive(Debug)]
pub(crate) struct ToolBarItem {
    data: Rc<ToolBarData>,
    identifier: Retained<NSString>,
    item: Retained<NSToolbarItem>,
}

impl ToolBarItem {
    pub(crate) fn new(parent: &ToolBar, view: &NSView, label: &str) -> Result<Self> {
        let data = parent.data();
        let mtm = data.mtm();

        catch(|| {
            let identifier = NSString::from_str(&format!(
                "winio.toolbar.item.{}",
                ITEM_ID.fetch_add(1, Ordering::Relaxed)
            ));
            let item = NSToolbarItem::initWithItemIdentifier(mtm.alloc(), &identifier);
            item.setView(Some(view));
            item.setAutovalidates(false);
            set_item_label(&item, label);

            data.register_item(&identifier, item.clone());
            let index = data.toolbar().items().len() as NSInteger;
            data.toolbar()
                .insertItemWithItemIdentifier_atIndex(&identifier, index);

            Ok(Self {
                data: data.clone(),
                identifier,
                item,
            })
        })
        .flatten()
    }

    pub(crate) fn set_label(&self, label: &str) -> Result<()> {
        catch(|| {
            set_item_label(&self.item, label);
            Ok(())
        })
        .flatten()
    }
}

impl Drop for ToolBarItem {
    fn drop(&mut self) {
        self.data
            .toolbar()
            .removeItemWithItemIdentifier(&self.identifier);
        self.data.unregister_item(&self.identifier);
    }
}

impl AsToolBarItem for ToolBarItem {
    fn as_tool_bar_item(&self) -> BorrowedToolBarItem<'_> {
        BorrowedToolBarItem::app_kit(&self.item)
    }
}

#[derive(Debug, Default)]
struct ToolBarActionDelegateIvars {
    action: Callback,
}

define_class! {
    #[unsafe(super(NSObject))]
    #[name = "WinioToolBarActionDelegate"]
    #[ivars = ToolBarActionDelegateIvars]
    #[thread_kind = MainThreadOnly]
    #[derive(Debug)]
    struct ToolBarActionDelegate;

    #[allow(non_snake_case)]
    impl ToolBarActionDelegate {
        #[unsafe(method_id(init))]
        fn init(this: Allocated<Self>) -> Option<Retained<Self>> {
            let this = this.set_ivars(ToolBarActionDelegateIvars::default());
            unsafe { msg_send![super(this), init] }
        }

        #[unsafe(method(onAction))]
        unsafe fn onAction(&self) {
            self.ivars().action.signal::<GlobalRuntime>(());
        }
    }
}

impl ToolBarActionDelegate {
    fn new(mtm: MainThreadMarker) -> Retained<Self> {
        unsafe { msg_send![mtm.alloc::<Self>(), init] }
    }

    async fn wait(&self) {
        self.ivars().action.wait().await;
    }
}

/// Click handling shared by all clickable toolbar items.
#[derive(Debug)]
pub(crate) struct ToolBarClick {
    delegate: Retained<ToolBarActionDelegate>,
}

impl ToolBarClick {
    pub(crate) fn new(mtm: MainThreadMarker, button: &NSButton) -> Self {
        let delegate = ToolBarActionDelegate::new(mtm);
        unsafe {
            button.setTarget(Some(&delegate));
            button.setAction(Some(sel!(onAction)));
        }
        Self { delegate }
    }

    pub(crate) async fn wait(&self) {
        self.delegate.wait().await;
    }
}

/// Shared implementation for plain, check and radio toolbar buttons.
#[derive(Debug)]
pub(crate) struct ToolBarButtonBase {
    item: ToolBarItem,
    button: Retained<NSButton>,
    click: ToolBarClick,
}

impl ToolBarButtonBase {
    pub(crate) fn new(parent: &ToolBar, button_type: NSButtonType) -> Result<Self> {
        let mtm = parent.data().mtm();

        catch(|| {
            let button = NSButton::new(mtm);
            button.setTitle(ns_string!(""));
            button.setBezelStyle(toolbar_bezel_style());
            button.setButtonType(button_type);
            button.setImageScaling(NSImageScaling::ScaleProportionallyDown);

            let click = ToolBarClick::new(mtm, &button);

            button.sizeToFit();
            let item = ToolBarItem::new(parent, &button, "")?;
            Ok(Self {
                item,
                button,
                click,
            })
        })
        .flatten()
    }

    pub(crate) fn is_enabled(&self) -> Result<bool> {
        catch(|| self.button.isEnabled())
    }

    pub(crate) fn set_enabled(&self, enabled: bool) -> Result<()> {
        catch(|| self.button.setEnabled(enabled))
    }

    pub(crate) fn text(&self) -> Result<String> {
        catch(|| from_nsstring(&self.button.title()))
    }

    pub(crate) fn set_text(&self, text: impl AsRef<str>) -> Result<()> {
        let text = text.as_ref();
        catch(|| self.button.setTitle(&NSString::from_str(text)))?;
        self.item.set_label(text)?;
        self.refresh_appearance()
    }

    pub(crate) fn set_icon(&self, icon: Option<&Image>) -> Result<()> {
        catch(|| {
            if let Some(icon) = icon {
                let image = icon.nsimage(Some(24.0))?;
                self.button.setImage(Some(&image));
            } else {
                self.button.setImage(None);
            }
            Ok(())
        })
        .flatten()?;
        self.refresh_appearance()
    }

    pub(crate) fn is_checked(&self) -> Result<bool> {
        catch(|| self.button.state() == NSControlStateValueOn)
    }

    pub(crate) fn set_checked(&self, checked: bool) -> Result<()> {
        catch(|| {
            self.button.setState(if checked {
                NSControlStateValueOn
            } else {
                NSControlStateValueOff
            })
        })
    }

    pub(crate) async fn wait_click(&self) {
        self.click.wait().await;
    }

    fn refresh_appearance(&self) -> Result<()> {
        catch(|| {
            refresh_image_position(&self.button);
            self.button.sizeToFit();
            Ok(())
        })
        .flatten()
    }
}

impl AsToolBarItem for ToolBarButtonBase {
    fn as_tool_bar_item(&self) -> BorrowedToolBarItem<'_> {
        self.item.as_tool_bar_item()
    }
}
