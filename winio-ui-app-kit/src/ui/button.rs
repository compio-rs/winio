use inherit_methods_macro::inherit_methods;
use objc2::{
    DeclaredClass, MainThreadOnly, define_class, msg_send,
    rc::{Allocated, Retained},
    sel,
};
use objc2_app_kit::{
    NSBezelStyle, NSButton, NSButtonType, NSControlStateValueOff, NSControlStateValueOn,
};
use objc2_foundation::{MainThreadMarker, NSObject, NSString};
use winio_callback::Callback;
use winio_handle::AsContainer;
use winio_primitive::{Point, Size};

use crate::{
    GlobalRuntime,
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
    pub fn new(parent: impl AsContainer) -> Self {
        unsafe {
            let parent = parent.as_container();
            let mtm = parent.mtm();

            let view = NSButton::new(mtm);
            let handle = Widget::from_nsview(&parent, Retained::cast_unchecked(view.clone()));

            let delegate = ButtonDelegate::new(mtm);
            view.setTarget(Some(&delegate));
            view.setAction(Some(sel!(onAction)));

            view.setBezelStyle(NSBezelStyle::FlexiblePush);
            Self {
                handle,
                view,
                delegate,
            }
        }
    }

    pub fn is_visible(&self) -> bool;

    pub fn set_visible(&mut self, v: bool);

    pub fn is_enabled(&self) -> bool;

    pub fn set_enabled(&mut self, v: bool);

    pub fn preferred_size(&self) -> Size;

    pub fn loc(&self) -> Point;

    pub fn set_loc(&mut self, p: Point);

    pub fn size(&self) -> Size;

    pub fn set_size(&mut self, v: Size);

    pub fn tooltip(&self) -> String;

    pub fn set_tooltip(&mut self, s: impl AsRef<str>);

    pub fn text(&self) -> String {
        unsafe { from_nsstring(&self.view.title()) }
    }

    pub fn set_text(&mut self, s: impl AsRef<str>) {
        unsafe {
            self.view.setTitle(&NSString::from_str(s.as_ref()));
        }
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
    pub fn new(parent: impl AsContainer) -> Self {
        let handle = Button::new(parent);
        unsafe {
            handle.view.setButtonType(NSButtonType::Switch);
            handle.view.setAllowsMixedState(false);
        }
        Self { handle }
    }

    pub fn is_visible(&self) -> bool;

    pub fn set_visible(&mut self, v: bool);

    pub fn is_enabled(&self) -> bool;

    pub fn set_enabled(&mut self, v: bool);

    pub fn preferred_size(&self) -> Size;

    pub fn loc(&self) -> Point;

    pub fn set_loc(&mut self, p: Point);

    pub fn size(&self) -> Size;

    pub fn set_size(&mut self, v: Size);

    pub fn tooltip(&self) -> String;

    pub fn set_tooltip(&mut self, s: impl AsRef<str>);

    pub fn text(&self) -> String;

    pub fn set_text(&mut self, s: impl AsRef<str>);

    pub fn is_checked(&self) -> bool {
        unsafe { self.handle.view.state() == NSControlStateValueOn }
    }

    pub fn set_checked(&mut self, v: bool) {
        unsafe {
            self.handle.view.setState(if v {
                NSControlStateValueOn
            } else {
                NSControlStateValueOff
            })
        }
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
    pub fn new(parent: impl AsContainer) -> Self {
        let handle = Button::new(parent);
        unsafe {
            handle.view.setButtonType(NSButtonType::Radio);
            handle.view.setAllowsMixedState(false);
        }
        Self { handle }
    }

    pub fn is_visible(&self) -> bool;

    pub fn set_visible(&mut self, v: bool);

    pub fn is_enabled(&self) -> bool;

    pub fn set_enabled(&mut self, v: bool);

    pub fn preferred_size(&self) -> Size;

    pub fn loc(&self) -> Point;

    pub fn set_loc(&mut self, p: Point);

    pub fn size(&self) -> Size;

    pub fn set_size(&mut self, v: Size);

    pub fn tooltip(&self) -> String;

    pub fn set_tooltip(&mut self, s: impl AsRef<str>);

    pub fn text(&self) -> String;

    pub fn set_text(&mut self, s: impl AsRef<str>);

    pub fn is_checked(&self) -> bool {
        unsafe { self.handle.view.state() == NSControlStateValueOn }
    }

    pub fn set_checked(&mut self, v: bool) {
        unsafe {
            self.handle.view.setState(if v {
                NSControlStateValueOn
            } else {
                NSControlStateValueOff
            })
        }
    }

    pub async fn wait_click(&self) {
        self.handle.wait_click().await
    }
}

winio_handle::impl_as_widget!(RadioButton, handle);

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
