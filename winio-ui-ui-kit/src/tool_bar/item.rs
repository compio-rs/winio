use std::{
    rc::{Rc, Weak},
    sync::atomic::{AtomicUsize, Ordering},
};

use objc2::{
    DeclaredClass, MainThreadMarker, MainThreadOnly, define_class, msg_send,
    rc::{Allocated, Retained},
    sel,
};
use objc2_foundation::{NSObject, NSString};
use objc2_ui_kit::{
    UIBarButtonItem, UIButton, UIButtonType, UIColor, UIControlEvents, UIControlState,
    UIImageRenderingMode,
};
use winio_callback::Callback;
use winio_handle::{AsToolBarItem, BorrowedToolBarItem};

use crate::{
    GlobalRuntime, Image, Result, catch,
    tool_bar::{ToolBar, ToolBarData},
};

static ITEM_ID: AtomicUsize = AtomicUsize::new(0);

#[derive(Debug)]
pub(crate) struct ToolBarItem {
    data: Weak<ToolBarData>,
    id: usize,
    item: Retained<UIBarButtonItem>,
}

impl ToolBarItem {
    pub(crate) fn new(parent: &ToolBar, item: Retained<UIBarButtonItem>) -> Result<Self> {
        let id = ITEM_ID.fetch_add(1, Ordering::Relaxed);
        let data = parent.data();
        data.add_item(id, item.clone());
        Ok(Self {
            data: Rc::downgrade(data),
            id,
            item,
        })
    }
}

impl Drop for ToolBarItem {
    fn drop(&mut self) {
        if let Some(data) = self.data.upgrade() {
            data.remove_item(self.id);
        }
    }
}

impl AsToolBarItem for ToolBarItem {
    fn as_tool_bar_item(&self) -> BorrowedToolBarItem<'_> {
        BorrowedToolBarItem::ui_kit(&self.item)
    }
}

#[derive(Debug, Default)]
struct ToolBarActionDelegateIvars {
    action: Callback,
}

define_class! {
    #[unsafe(super(NSObject))]
    #[name = "WinioToolBarActionDelegateUIKit"]
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

#[derive(Debug)]
pub(crate) struct ToolBarClick {
    delegate: Retained<ToolBarActionDelegate>,
}

impl ToolBarClick {
    pub(crate) fn new(mtm: MainThreadMarker, button: &UIButton) -> Self {
        let delegate = ToolBarActionDelegate::new(mtm);
        unsafe {
            button.addTarget_action_forControlEvents(
                Some(&delegate),
                sel!(onAction),
                UIControlEvents::TouchUpInside,
            );
        }
        Self { delegate }
    }

    pub(crate) async fn wait(&self) {
        self.delegate.wait().await;
    }
}

fn update_tint(button: &UIButton, checked: bool) {
    let color = if checked {
        UIColor::systemBlueColor()
    } else {
        UIColor::labelColor()
    };
    unsafe { button.setTintColor(Some(&color)) };
}

#[derive(Debug)]
pub(crate) struct ToolBarButtonBase {
    item: ToolBarItem,
    button: Retained<UIButton>,
    click: ToolBarClick,
    toggle: bool,
}

impl ToolBarButtonBase {
    pub(crate) fn new(parent: &ToolBar, toggle: bool, auto_toggle: bool) -> Result<Self> {
        let mtm = parent.data().mtm();

        catch(|| {
            let button = UIButton::buttonWithType(UIButtonType::System, mtm);
            if toggle {
                button.setChangesSelectionAsPrimaryAction(auto_toggle);
                update_tint(&button, false);
            }
            let click = ToolBarClick::new(mtm, &button);

            let bar_item = UIBarButtonItem::initWithCustomView(mtm.alloc(), &button);
            let item = ToolBarItem::new(parent, bar_item)?;
            Ok(Self {
                item,
                button,
                click,
                toggle,
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
        catch(|| {
            self.button
                .titleForState(UIControlState::Normal)
                .as_deref()
                .map(crate::from_nsstring)
                .unwrap_or_default()
        })
    }

    pub(crate) fn set_text(&self, text: impl AsRef<str>) -> Result<()> {
        let title = NSString::from_str(text.as_ref());
        catch(|| {
            self.button
                .setTitle_forState(Some(&title), UIControlState::Normal);
            self.button.invalidateIntrinsicContentSize();
        })
    }

    pub(crate) fn set_icon(&self, icon: Option<&Image>) -> Result<()> {
        catch(|| {
            if let Some(icon) = icon {
                let image = icon.uiimage(Some(24.0))?;
                let image = image.imageWithRenderingMode(UIImageRenderingMode::AlwaysTemplate);
                self.button
                    .setImage_forState(Some(&image), UIControlState::Normal);
            } else {
                self.button.setImage_forState(None, UIControlState::Normal);
            }
            self.button.invalidateIntrinsicContentSize();
            Ok(())
        })
        .flatten()
    }

    pub(crate) fn is_checked(&self) -> Result<bool> {
        catch(|| self.button.isSelected())
    }

    pub(crate) fn set_checked(&self, checked: bool) -> Result<()> {
        catch(|| {
            self.button.setSelected(checked);
            if self.toggle {
                update_tint(&self.button, checked);
            }
        })
    }

    pub(crate) async fn wait_click(&self) {
        self.click.wait().await;
        if self.toggle {
            update_tint(&self.button, self.button.isSelected());
        }
    }
}

impl AsToolBarItem for ToolBarButtonBase {
    fn as_tool_bar_item(&self) -> BorrowedToolBarItem<'_> {
        self.item.as_tool_bar_item()
    }
}
