use objc2::{
    DeclaredClass, MainThreadOnly, define_class, msg_send,
    rc::{Allocated, Retained},
    sel,
};
use objc2_app_kit::{
    NSBezelStyle, NSButton, NSButtonType, NSControlStateValueOff, NSControlStateValueOn,
};
use objc2_foundation::{MainThreadMarker, NSObject, NSString};

use crate::{
    AsWindow, Point, Size,
    ui::{Callback, Widget, from_nsstring},
};

#[derive(Debug)]
pub struct Button {
    handle: Widget,
    view: Retained<NSButton>,
    delegate: Retained<ButtonDelegate>,
}

impl Button {
    pub fn new(parent: impl AsWindow) -> Self {
        unsafe {
            let mtm = MainThreadMarker::new().unwrap();

            let view = NSButton::new(mtm);
            let handle = Widget::from_nsview(parent, Retained::cast_unchecked(view.clone()));

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

    pub fn is_visible(&self) -> bool {
        self.handle.is_visible()
    }

    pub fn set_visible(&mut self, v: bool) {
        self.handle.set_visible(v);
    }

    pub fn is_enabled(&self) -> bool {
        self.handle.is_enabled()
    }

    pub fn set_enabled(&mut self, v: bool) {
        self.handle.set_enabled(v);
    }

    pub fn preferred_size(&self) -> Size {
        self.handle.preferred_size()
    }

    pub fn loc(&self) -> Point {
        self.handle.loc()
    }

    pub fn set_loc(&mut self, p: Point) {
        self.handle.set_loc(p)
    }

    pub fn size(&self) -> Size {
        self.handle.size()
    }

    pub fn set_size(&mut self, v: Size) {
        self.handle.set_size(v)
    }

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

#[derive(Debug)]
pub struct CheckBox {
    handle: Button,
}

impl CheckBox {
    pub fn new(parent: impl AsWindow) -> Self {
        let handle = Button::new(parent);
        unsafe {
            handle.view.setButtonType(NSButtonType::Switch);
            handle.view.setAllowsMixedState(false);
        }
        Self { handle }
    }

    pub fn is_visible(&self) -> bool {
        self.handle.is_visible()
    }

    pub fn set_visible(&mut self, v: bool) {
        self.handle.set_visible(v);
    }

    pub fn is_enabled(&self) -> bool {
        self.handle.is_enabled()
    }

    pub fn set_enabled(&mut self, v: bool) {
        self.handle.set_enabled(v);
    }

    pub fn preferred_size(&self) -> Size {
        self.handle.preferred_size()
    }

    pub fn loc(&self) -> Point {
        self.handle.loc()
    }

    pub fn set_loc(&mut self, p: Point) {
        self.handle.set_loc(p)
    }

    pub fn size(&self) -> Size {
        self.handle.size()
    }

    pub fn set_size(&mut self, v: Size) {
        self.handle.set_size(v)
    }

    pub fn text(&self) -> String {
        self.handle.text()
    }

    pub fn set_text(&mut self, s: impl AsRef<str>) {
        self.handle.set_text(s);
    }

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

#[derive(Debug)]
pub struct RadioButton {
    handle: Button,
}

impl RadioButton {
    pub fn new(parent: impl AsWindow) -> Self {
        let handle = Button::new(parent);
        unsafe {
            handle.view.setButtonType(NSButtonType::Radio);
            handle.view.setAllowsMixedState(false);
        }
        Self { handle }
    }

    pub fn is_visible(&self) -> bool {
        self.handle.is_visible()
    }

    pub fn set_visible(&mut self, v: bool) {
        self.handle.set_visible(v);
    }

    pub fn is_enabled(&self) -> bool {
        self.handle.is_enabled()
    }

    pub fn set_enabled(&mut self, v: bool) {
        self.handle.set_enabled(v);
    }

    pub fn preferred_size(&self) -> Size {
        self.handle.preferred_size()
    }

    pub fn loc(&self) -> Point {
        self.handle.loc()
    }

    pub fn set_loc(&mut self, p: Point) {
        self.handle.set_loc(p)
    }

    pub fn size(&self) -> Size {
        self.handle.size()
    }

    pub fn set_size(&mut self, v: Size) {
        self.handle.set_size(v)
    }

    pub fn text(&self) -> String {
        self.handle.text()
    }

    pub fn set_text(&mut self, s: impl AsRef<str>) {
        self.handle.set_text(s);
    }

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

#[derive(Debug, Default, Clone)]
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
            self.ivars().action.signal(());
        }
    }
}

impl ButtonDelegate {
    pub fn new(mtm: MainThreadMarker) -> Retained<Self> {
        unsafe { msg_send![mtm.alloc::<Self>(), init] }
    }
}
