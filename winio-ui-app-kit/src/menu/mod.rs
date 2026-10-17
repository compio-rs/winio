use inherit_methods_macro::inherit_methods;
use objc2::{
    DefinedClass, MainThreadMarker, MainThreadOnly, available, define_class, msg_send,
    rc::{Allocated, Retained},
    sel,
};
use objc2_app_kit::{
    NSApplication, NSControlStateValueOff, NSControlStateValueOn, NSMenu, NSMenuItem,
};
use objc2_foundation::{NSInteger, NSObject, NSString};
use winio_callback::Callback;
use winio_handle::{
    AsMenuBar, AsMenuContainer, AsMenuItem, AsWindow, BorrowedMenuBar, BorrowedMenuContainer,
    BorrowedMenuItem,
};
use winio_pollable::GlobalRuntime;

use crate::{Error, Image, Result, catch, from_nsstring};

#[derive(Debug)]
struct MenuBase {
    inner: Retained<NSMenu>,
}

impl MenuBase {
    pub fn new() -> Result<Self> {
        let mtm = MainThreadMarker::new().ok_or_else(|| Error::NotMainThread)?;
        catch(|| {
            let inner = NSMenu::new(mtm);
            Self { inner }
        })
    }
}

impl AsMenuBar for MenuBase {
    fn as_menu_bar(&self) -> BorrowedMenuBar<'_> {
        BorrowedMenuBar::app_kit(&self.inner)
    }
}

impl AsMenuContainer for MenuBase {
    fn as_menu_container(&self) -> BorrowedMenuContainer<'_> {
        BorrowedMenuContainer::app_kit(&self.inner)
    }
}

#[derive(Debug)]
pub struct MenuBar {
    inner: MenuBase,
}

impl MenuBar {
    pub fn new(_parent: impl AsWindow) -> Result<Self> {
        let mtm = MainThreadMarker::new().ok_or(Error::NotMainThread)?;
        let app = NSApplication::sharedApplication(mtm);
        if catch(|| app.mainMenu().is_some())? {
            return Err(Error::MenuBarExists);
        }
        let inner = MenuBase::new()?;
        catch(|| {
            app.setMainMenu(Some(&inner.inner));
        })?;
        Ok(Self { inner })
    }

    pub fn is_visible(&self) -> Result<bool> {
        catch(|| {
            let app = NSApplication::sharedApplication(self.inner.inner.mtm());
            if let Some(menu) = app.mainMenu() {
                menu == self.inner.inner
            } else {
                false
            }
        })
    }

    pub fn set_visible(&mut self, visible: bool) -> Result<()> {
        catch(|| {
            let app = NSApplication::sharedApplication(self.inner.inner.mtm());
            if visible {
                app.setMainMenu(Some(&self.inner.inner));
            } else {
                app.setMainMenu(None);
            }
        })
    }
}

winio_handle::impl_as_menu_bar!(MenuBar, inner);

#[derive(Debug)]
struct MenuItemBase {
    inner: Retained<NSMenuItem>,
    delegate: Retained<MenuItemDelegate>,
}

impl MenuItemBase {
    pub fn new(parent: &NSMenu) -> Result<Self> {
        catch(|| {
            let inner = NSMenuItem::new(parent.mtm());

            let delegate = MenuItemDelegate::new(parent.mtm());
            unsafe {
                inner.setTarget(Some(&delegate));
                inner.setAction(Some(sel!(onAction)));
            }

            parent.addItem(&inner);
            Self { inner, delegate }
        })
    }

    pub fn is_enabled(&self) -> Result<bool> {
        catch(|| self.inner.isEnabled())
    }

    pub fn set_enabled(&mut self, enabled: bool) -> Result<()> {
        catch(|| self.inner.setEnabled(enabled))
    }

    pub fn text(&self) -> Result<String> {
        catch(|| from_nsstring(&self.inner.title()))
    }

    pub fn set_text(&mut self, s: impl AsRef<str>) -> Result<()> {
        catch(|| {
            self.inner.setTitle(&NSString::from_str(s.as_ref()));
        })
    }

    pub fn set_icon(&mut self, icon: Option<&Image>) -> Result<()> {
        catch(|| {
            if let Some(icon) = icon {
                let image = icon.nsimage(Some(16.0))?;
                self.inner.setImage(Some(&image));

                #[allow(non_upper_case_globals)]
                const NSMenuItemImageVisibilityVisible: NSInteger = 1;

                if available!(macos = 27.0) {
                    unsafe {
                        let () = msg_send![&self.inner, setPreferredImageVisibility: NSMenuItemImageVisibilityVisible];
                    }
                }
            } else {
                self.inner.setImage(None);
            }
            Ok(())
        })
        .flatten()
    }

    pub fn is_checked(&self) -> Result<bool> {
        catch(|| self.inner.state() == NSControlStateValueOn)
    }

    pub fn set_checked(&self, checked: bool) -> Result<()> {
        catch(|| {
            let state = if checked {
                NSControlStateValueOn
            } else {
                NSControlStateValueOff
            };
            self.inner.setState(state);
        })
    }

    pub async fn wait_click(&self) {
        self.delegate.ivars().action.wait().await;
    }
}

impl AsMenuItem for MenuItemBase {
    fn as_menu_item(&self) -> BorrowedMenuItem<'_> {
        BorrowedMenuItem::app_kit(&self.inner)
    }
}

#[derive(Debug, Default)]
struct MenuItemDelegateIvars {
    action: Callback,
}

define_class! {
    #[unsafe(super(NSObject))]
    #[name = "WinioMenuItemDelegate"]
    #[ivars = MenuItemDelegateIvars]
    #[thread_kind = MainThreadOnly]
    #[derive(Debug)]
    struct MenuItemDelegate;

    #[allow(non_snake_case)]
    impl MenuItemDelegate {
        #[unsafe(method_id(init))]
        fn init(this: Allocated<Self>) -> Option<Retained<Self>> {
            let this = this.set_ivars(MenuItemDelegateIvars::default());
            unsafe { msg_send![super(this), init] }
        }

        #[unsafe(method(onAction))]
        unsafe fn onAction(&self) {
            self.ivars().action.signal::<GlobalRuntime>(());
        }
    }
}

impl MenuItemDelegate {
    pub fn new(mtm: MainThreadMarker) -> Retained<Self> {
        unsafe { msg_send![mtm.alloc::<Self>(), init] }
    }
}

#[derive(Debug)]
pub struct MenuBarItem {
    inner: MenuItemBase,
    menu: MenuBase,
}

#[inherit_methods(from = "self.inner")]
impl MenuBarItem {
    pub fn new(parent: impl AsMenuBar) -> Result<Self> {
        let parent = parent.as_menu_bar().as_app_kit();
        let inner = MenuItemBase::new(parent)?;
        let menu = MenuBase::new()?;
        catch(|| {
            inner.inner.setSubmenu(Some(&menu.inner));
        })?;
        Ok(Self { inner, menu })
    }

    pub fn is_enabled(&self) -> Result<bool>;

    pub fn set_enabled(&mut self, enabled: bool) -> Result<()>;

    pub fn text(&self) -> Result<String>;

    pub fn set_text(&mut self, s: impl AsRef<str>) -> Result<()>;
}

winio_handle::impl_as_menu_container!(MenuBarItem, menu);

#[derive(Debug)]
pub struct MenuItem {
    inner: MenuItemBase,
}

#[inherit_methods(from = "self.inner")]
impl MenuItem {
    pub fn new(parent: impl AsMenuContainer) -> Result<Self> {
        let parent = parent.as_menu_container().as_app_kit();
        let inner = MenuItemBase::new(parent)?;
        Ok(Self { inner })
    }

    pub fn is_enabled(&self) -> Result<bool>;

    pub fn set_enabled(&mut self, enabled: bool) -> Result<()>;

    pub fn text(&self) -> Result<String>;

    pub fn set_text(&mut self, s: impl AsRef<str>) -> Result<()>;

    pub fn set_icon(&mut self, icon: Option<&Image>) -> Result<()>;

    pub async fn wait_click(&self) {
        self.inner.wait_click().await
    }
}

winio_handle::impl_as_menu_item!(MenuItem, inner);

#[derive(Debug)]
pub struct PopupMenuItem {
    inner: MenuItemBase,
    menu: MenuBase,
}

#[inherit_methods(from = "self.inner")]
impl PopupMenuItem {
    pub fn new(parent: impl AsMenuContainer) -> Result<Self> {
        let parent = parent.as_menu_container().as_app_kit();
        let inner = MenuItemBase::new(parent)?;
        let menu = MenuBase::new()?;
        catch(|| {
            inner.inner.setSubmenu(Some(&menu.inner));
        })?;
        Ok(Self { inner, menu })
    }

    pub fn is_enabled(&self) -> Result<bool>;

    pub fn set_enabled(&mut self, enabled: bool) -> Result<()>;

    pub fn text(&self) -> Result<String>;

    pub fn set_text(&mut self, s: impl AsRef<str>) -> Result<()>;

    pub fn set_icon(&mut self, icon: Option<&Image>) -> Result<()>;
}

winio_handle::impl_as_menu_container!(PopupMenuItem, menu);
winio_handle::impl_as_menu_item!(PopupMenuItem, inner);

#[derive(Debug)]
pub struct CheckMenuItem {
    inner: MenuItemBase,
}

#[inherit_methods(from = "self.inner")]
impl CheckMenuItem {
    pub fn new(parent: impl AsMenuContainer) -> Result<Self> {
        let parent = parent.as_menu_container().as_app_kit();
        let inner = MenuItemBase::new(parent)?;
        Ok(Self { inner })
    }

    pub fn is_enabled(&self) -> Result<bool>;

    pub fn set_enabled(&mut self, enabled: bool) -> Result<()>;

    pub fn text(&self) -> Result<String>;

    pub fn set_text(&mut self, s: impl AsRef<str>) -> Result<()>;

    pub fn is_checked(&self) -> Result<bool>;

    pub fn set_checked(&mut self, checked: bool) -> Result<()>;

    pub async fn wait_click(&self) {
        self.inner.wait_click().await;
        if let Ok(checked) = self.is_checked() {
            self.inner.set_checked(!checked).ok();
        }
    }
}

winio_handle::impl_as_menu_item!(CheckMenuItem, inner);

#[derive(Debug)]
pub struct RadioMenuItem {
    inner: MenuItemBase,
}

#[inherit_methods(from = "self.inner")]
impl RadioMenuItem {
    pub fn new(parent: impl AsMenuContainer) -> Result<Self> {
        let parent = parent.as_menu_container().as_app_kit();
        let inner = MenuItemBase::new(parent)?;
        Ok(Self { inner })
    }

    pub fn is_enabled(&self) -> Result<bool>;

    pub fn set_enabled(&mut self, enabled: bool) -> Result<()>;

    pub fn text(&self) -> Result<String>;

    pub fn set_text(&mut self, s: impl AsRef<str>) -> Result<()>;

    pub fn is_checked(&self) -> Result<bool>;

    pub fn set_checked(&mut self, checked: bool) -> Result<()>;

    pub async fn wait_click(&self) {
        self.inner.wait_click().await;
        self.inner.set_checked(true).ok();
    }
}

winio_handle::impl_as_menu_item!(RadioMenuItem, inner);

#[derive(Debug)]
pub struct SeparatorMenuItem {
    inner: Retained<NSMenuItem>,
}

impl SeparatorMenuItem {
    pub fn new(parent: impl AsMenuContainer) -> Result<Self> {
        catch(|| {
            let parent = parent.as_menu_container().as_app_kit();
            let inner = NSMenuItem::separatorItem(parent.mtm());
            parent.addItem(&inner);
            Self { inner }
        })
    }
}

impl AsMenuItem for SeparatorMenuItem {
    fn as_menu_item(&self) -> BorrowedMenuItem<'_> {
        BorrowedMenuItem::app_kit(&self.inner)
    }
}
