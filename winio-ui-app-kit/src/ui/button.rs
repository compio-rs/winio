use compio_log::{error, info, warn};
use inherit_methods_macro::inherit_methods;
use objc2::{
    DeclaredClass, MainThreadOnly, define_class, msg_send,
    rc::{Allocated, Retained},
    sel,
};
use objc2_app_kit::{
    NSBezelStyle, NSButton, NSButtonType, NSControlStateValueOff, NSControlStateValueOn,
    NSWorkspace,
};
use objc2_foundation::{MainThreadMarker, NSObject, NSString, NSURL};
use winio_callback::Callback;
use winio_handle::AsContainer;
use winio_primitive::{Point, Size};

use crate::{
    GlobalRuntime, Result, catch,
    ui::{Widget, from_nsstring},
};

#[derive(Debug)]
pub struct Button {
    handle: Widget,
    view: Retained<NSButton>,
    delegate: Retained<ButtonDelegate>,
}

#[inherit_methods(from = "self.handle")]
impl Button {
    pub fn new(parent: impl AsContainer) -> Result<Self> {
        let parent = parent.as_container();
        let mtm = parent.as_app_kit().mtm();

        catch(|| unsafe {
            let view = NSButton::new(mtm);
            let handle = Widget::from_nsview(parent, Retained::cast_unchecked(view.clone()))?;

            let delegate = ButtonDelegate::new(mtm);
            view.setTarget(Some(&delegate));
            view.setAction(Some(sel!(onAction)));

            view.setBezelStyle(NSBezelStyle::FlexiblePush);
            Ok(Self {
                handle,
                view,
                delegate,
            })
        })
        .flatten()
    }

    pub fn is_visible(&self) -> Result<bool>;

    pub fn set_visible(&mut self, v: bool) -> Result<()>;

    pub fn is_enabled(&self) -> Result<bool>;

    pub fn set_enabled(&mut self, v: bool) -> Result<()>;

    pub fn preferred_size(&self) -> Result<Size>;

    pub fn loc(&self) -> Result<Point>;

    pub fn set_loc(&mut self, p: Point) -> Result<()>;

    pub fn size(&self) -> Result<Size>;

    pub fn set_size(&mut self, v: Size) -> Result<()>;

    pub fn tooltip(&self) -> Result<String>;

    pub fn set_tooltip(&mut self, s: impl AsRef<str>) -> Result<()>;

    pub fn text(&self) -> Result<String> {
        catch(|| from_nsstring(&self.view.title()))
    }

    pub fn set_text(&mut self, s: impl AsRef<str>) -> Result<()> {
        catch(|| self.view.setTitle(&NSString::from_str(s.as_ref())))
    }

    pub async fn wait_click(&self) {
        self.delegate.ivars().action.wait().await
    }
}

winio_handle::impl_as_widget!(Button, handle);

#[derive(Debug)]
pub struct CheckBox {
    handle: Button,
}

#[inherit_methods(from = "self.handle")]
impl CheckBox {
    pub fn new(parent: impl AsContainer) -> Result<Self> {
        let handle = Button::new(parent)?;
        catch(|| {
            handle.view.setButtonType(NSButtonType::Switch);
            handle.view.setAllowsMixedState(false);
        })?;
        Ok(Self { handle })
    }

    pub fn is_visible(&self) -> Result<bool>;

    pub fn set_visible(&mut self, v: bool) -> Result<()>;

    pub fn is_enabled(&self) -> Result<bool>;

    pub fn set_enabled(&mut self, v: bool) -> Result<()>;

    pub fn preferred_size(&self) -> Result<Size> {
        let mut s = self.handle.preferred_size()?;
        s.width += 4.0;
        Ok(s)
    }

    pub fn loc(&self) -> Result<Point>;

    pub fn set_loc(&mut self, p: Point) -> Result<()>;

    pub fn size(&self) -> Result<Size>;

    pub fn set_size(&mut self, v: Size) -> Result<()>;

    pub fn tooltip(&self) -> Result<String>;

    pub fn set_tooltip(&mut self, s: impl AsRef<str>) -> Result<()>;

    pub fn text(&self) -> Result<String>;

    pub fn set_text(&mut self, s: impl AsRef<str>) -> Result<()>;

    pub fn is_checked(&self) -> Result<bool> {
        catch(|| self.handle.view.state() == NSControlStateValueOn)
    }

    pub fn set_checked(&mut self, v: bool) -> Result<()> {
        catch(|| {
            self.handle.view.setState(if v {
                NSControlStateValueOn
            } else {
                NSControlStateValueOff
            })
        })
    }

    pub async fn wait_click(&self) {
        self.handle.wait_click().await
    }
}

winio_handle::impl_as_widget!(CheckBox, handle);

#[derive(Debug)]
pub struct RadioButton {
    handle: Button,
}

#[inherit_methods(from = "self.handle")]
impl RadioButton {
    pub fn new(parent: impl AsContainer) -> Result<Self> {
        let handle = Button::new(parent)?;
        catch(|| {
            handle.view.setButtonType(NSButtonType::Radio);
            handle.view.setAllowsMixedState(false);
        })?;
        Ok(Self { handle })
    }

    pub fn is_visible(&self) -> Result<bool>;

    pub fn set_visible(&mut self, v: bool) -> Result<()>;

    pub fn is_enabled(&self) -> Result<bool>;

    pub fn set_enabled(&mut self, v: bool) -> Result<()>;

    pub fn preferred_size(&self) -> Result<Size>;

    pub fn loc(&self) -> Result<Point>;

    pub fn set_loc(&mut self, p: Point) -> Result<()>;

    pub fn size(&self) -> Result<Size>;

    pub fn set_size(&mut self, v: Size) -> Result<()>;

    pub fn tooltip(&self) -> Result<String>;

    pub fn set_tooltip(&mut self, s: impl AsRef<str>) -> Result<()>;

    pub fn text(&self) -> Result<String>;

    pub fn set_text(&mut self, s: impl AsRef<str>) -> Result<()>;

    pub fn is_checked(&self) -> Result<bool> {
        catch(|| self.handle.view.state() == NSControlStateValueOn)
    }

    pub fn set_checked(&mut self, v: bool) -> Result<()> {
        catch(|| {
            self.handle.view.setState(if v {
                NSControlStateValueOn
            } else {
                NSControlStateValueOff
            })
        })
    }

    pub async fn wait_click(&self) {
        self.handle.wait_click().await
    }
}

winio_handle::impl_as_widget!(RadioButton, handle);

#[derive(Debug)]
pub struct LinkLabel {
    handle: Button,
    uri: String,
}

#[inherit_methods(from = "self.handle")]
impl LinkLabel {
    pub fn new(parent: impl AsContainer) -> Result<Self> {
        let handle = Button::new(parent)?;
        catch(|| {
            handle.view.setBordered(false);
            handle.view.setBezelStyle(NSBezelStyle::Badge);
        })?;
        Ok(Self {
            handle,
            uri: String::new(),
        })
    }

    pub fn is_visible(&self) -> Result<bool>;

    pub fn set_visible(&mut self, v: bool) -> Result<()>;

    pub fn is_enabled(&self) -> Result<bool>;

    pub fn set_enabled(&mut self, v: bool) -> Result<()>;

    pub fn preferred_size(&self) -> Result<Size>;

    pub fn loc(&self) -> Result<Point>;

    pub fn set_loc(&mut self, p: Point) -> Result<()>;

    pub fn size(&self) -> Result<Size>;

    pub fn set_size(&mut self, v: Size) -> Result<()>;

    pub fn tooltip(&self) -> Result<String>;

    pub fn set_tooltip(&mut self, s: impl AsRef<str>) -> Result<()>;

    pub fn text(&self) -> Result<String>;

    pub fn set_text(&mut self, s: impl AsRef<str>) -> Result<()>;

    pub fn uri(&self) -> Result<String> {
        Ok(self.uri.clone())
    }

    pub fn set_uri(&mut self, uri: impl AsRef<str>) -> Result<()> {
        self.uri = uri.as_ref().to_string();
        Ok(())
    }

    pub async fn wait_click(&self) {
        loop {
            self.handle.wait_click().await;
            if self.uri.is_empty() {
                break;
            } else {
                if let Some(url) = NSURL::URLWithString(&NSString::from_str(&self.uri)) {
                    info!("Opening link: {}", self.uri);
                    let opened = NSWorkspace::sharedWorkspace().openURL(&url);
                    if !opened {
                        error!("Failed to open link: {}", self.uri);
                    }
                } else {
                    warn!("Invalid URL: {}", self.uri);
                }
            }
        }
    }
}

winio_handle::impl_as_widget!(LinkLabel, handle);

#[derive(Debug, Default)]
struct ButtonDelegateIvars {
    action: Callback,
}

define_class! {
    #[unsafe(super(NSObject))]
    #[name = "WinioButtonDelegate"]
    #[ivars = ButtonDelegateIvars]
    #[thread_kind = MainThreadOnly]
    #[derive(Debug)]
    struct ButtonDelegate;

    #[allow(non_snake_case)]
    impl ButtonDelegate {
        #[unsafe(method_id(init))]
        fn init(this: Allocated<Self>) -> Option<Retained<Self>> {
            let this = this.set_ivars(ButtonDelegateIvars::default());
            unsafe { msg_send![super(this), init] }
        }

        #[unsafe(method(onAction))]
        unsafe fn onAction(&self) {
            self.ivars().action.signal::<GlobalRuntime>(());
        }
    }
}

impl ButtonDelegate {
    pub fn new(mtm: MainThreadMarker) -> Retained<Self> {
        unsafe { msg_send![mtm.alloc::<Self>(), init] }
    }
}
