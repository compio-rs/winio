use objc2::{
    ClassType, DeclaredClass, declare_class, msg_send_id,
    mutability::MainThreadOnly,
    rc::{Allocated, Id},
    runtime::ProtocolObject,
};
use objc2_app_kit::{
    NSControlTextEditingDelegate, NSTextAlignment, NSTextField, NSTextFieldDelegate,
};
use objc2_foundation::{MainThreadMarker, NSNotification, NSObject, NSObjectProtocol, NSString};

use crate::{
    AsRawWindow, AsWindow, HAlign, Point, Size,
    ui::{Callback, Widget, from_nsstring},
};

#[derive(Debug)]
pub struct Edit {
    handle: Widget,
    view: Id<NSTextField>,
    delegate: Id<EditDelegate>,
}

impl Edit {
    pub fn new(parent: impl AsWindow) -> Self {
        unsafe {
            let mtm = MainThreadMarker::new().unwrap();

            let view = NSTextField::new(mtm);
            view.setBezeled(true);
            view.setDrawsBackground(true);
            view.setEditable(true);
            view.setSelectable(true);

            let handle =
                Widget::from_nsview(parent.as_window().as_raw_window(), Id::cast(view.clone()));

            let delegate = EditDelegate::new(mtm);
            let del_obj = ProtocolObject::from_id(delegate.clone());
            view.setDelegate(Some(&del_obj));
            Self {
                handle,
                view,
                delegate,
            }
        }
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
        unsafe { from_nsstring(&self.view.stringValue()) }
    }

    pub fn set_text(&mut self, s: impl AsRef<str>) {
        unsafe {
            self.view.setStringValue(&NSString::from_str(s.as_ref()));
            self.view.sizeToFit();
        }
    }

    pub fn halign(&self) -> HAlign {
        let align = unsafe { self.view.alignment() };
        match align {
            NSTextAlignment::Right => HAlign::Right,
            NSTextAlignment::Center => HAlign::Center,
            _ => HAlign::Left,
        }
    }

    pub fn set_halign(&mut self, align: HAlign) {
        unsafe {
            let align = match align {
                HAlign::Left => NSTextAlignment::Left,
                HAlign::Center => NSTextAlignment::Center,
                HAlign::Right => NSTextAlignment::Right,
            };
            self.view.setAlignment(align);
        }
    }

    pub async fn wait_change(&self) {
        self.delegate.ivars().changed.wait().await
    }
}

#[derive(Default, Clone)]
struct EditDelegateIvars {
    changed: Callback,
}

declare_class! {
    #[derive(Debug)]
    struct EditDelegate;

    unsafe impl ClassType for EditDelegate {
        type Super = NSObject;
        type Mutability = MainThreadOnly;
        const NAME: &'static str = "WinioEditDelegate";
    }

    impl DeclaredClass for EditDelegate {
        type Ivars = EditDelegateIvars;
    }

    #[allow(non_snake_case)]
    unsafe impl EditDelegate {
        #[method_id(init)]
        fn init(this: Allocated<Self>) -> Option<Id<Self>> {
            let this = this.set_ivars(EditDelegateIvars::default());
            unsafe { msg_send_id![super(this), init] }
        }
    }

    unsafe impl NSObjectProtocol for EditDelegate {}

    #[allow(non_snake_case)]
    unsafe impl NSControlTextEditingDelegate for EditDelegate {
        #[method(controlTextDidChange:)]
        fn controlTextDidChange(&self, _notification: &NSNotification) {
            self.ivars().changed.signal(());
        }
    }

    unsafe impl NSTextFieldDelegate for EditDelegate {}
}

impl EditDelegate {
    pub fn new(mtm: MainThreadMarker) -> Id<Self> {
        unsafe { msg_send_id![mtm.alloc::<Self>(), init] }
    }
}
