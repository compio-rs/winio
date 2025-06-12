use objc2::{
    DeclaredClass, MainThreadOnly, define_class, msg_send,
    rc::{Allocated, Retained},
    runtime::ProtocolObject,
};
use objc2_app_kit::{
    NSControlTextEditingDelegate, NSSecureTextField, NSTextAlignment, NSTextField,
    NSTextFieldDelegate,
};
use objc2_foundation::{MainThreadMarker, NSNotification, NSObject, NSObjectProtocol, NSString};

use crate::{
    AsWindow, HAlign, Point, Size,
    ui::{Callback, Widget, from_nsstring},
};

#[derive(Debug)]
pub struct Edit {
    handle: Widget,
    phandle: Widget,
    view: Retained<NSTextField>,
    pview: Retained<NSTextField>,
    password: bool,
    delegate: Retained<EditDelegate>,
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

            let pview: Retained<NSTextField> =
                Retained::cast_unchecked(NSSecureTextField::new(mtm));
            pview.setBezeled(true);
            pview.setDrawsBackground(true);
            pview.setEditable(true);
            pview.setSelectable(true);
            pview.setHidden(true);

            let handle =
                Widget::from_nsview(parent.as_window(), Retained::cast_unchecked(view.clone()));
            let phandle =
                Widget::from_nsview(parent.as_window(), Retained::cast_unchecked(pview.clone()));

            let delegate = EditDelegate::new(mtm);
            let del_obj = ProtocolObject::from_retained(delegate.clone());
            view.setDelegate(Some(&del_obj));
            pview.setDelegate(Some(&del_obj));
            Self {
                handle,
                phandle,
                view,
                pview,
                password: false,
                delegate,
            }
        }
    }

    pub fn is_visible(&self) -> bool {
        if self.password {
            &self.phandle
        } else {
            &self.handle
        }
        .is_visible()
    }

    pub fn set_visible(&mut self, v: bool) {
        if self.password {
            &mut self.phandle
        } else {
            &mut self.handle
        }
        .set_visible(v);
    }

    pub fn is_enabled(&self) -> bool {
        self.handle.is_enabled()
    }

    pub fn set_enabled(&mut self, v: bool) {
        self.handle.set_enabled(v);
        self.phandle.set_enabled(v);
    }

    pub fn preferred_size(&self) -> Size {
        self.handle.preferred_size()
    }

    pub fn loc(&self) -> Point {
        self.handle.loc()
    }

    pub fn set_loc(&mut self, p: Point) {
        self.handle.set_loc(p);
        self.phandle.set_loc(p);
    }

    pub fn size(&self) -> Size {
        self.handle.size()
    }

    pub fn set_size(&mut self, v: Size) {
        self.handle.set_size(v);
        self.phandle.set_size(v);
    }

    pub fn text(&self) -> String {
        unsafe {
            from_nsstring(
                &if self.password {
                    &self.pview
                } else {
                    &self.view
                }
                .stringValue(),
            )
        }
    }

    pub fn set_text(&mut self, s: impl AsRef<str>) {
        unsafe {
            if self.password {
                &self.pview
            } else {
                &self.view
            }
            .setStringValue(&NSString::from_str(s.as_ref()));
        }
    }

    pub fn is_password(&self) -> bool {
        self.password
    }

    pub fn set_password(&mut self, v: bool) {
        if self.password != v {
            unsafe {
                if v {
                    self.pview.setStringValue(&self.view.stringValue());
                } else {
                    self.view.setStringValue(&self.pview.stringValue());
                }
            }
            self.password = v;
            self.pview.setHidden(!v);
            self.view.setHidden(v);
        }
    }

    pub fn halign(&self) -> HAlign {
        let align = unsafe { self.view.alignment() };
        match align {
            NSTextAlignment::Right => HAlign::Right,
            NSTextAlignment::Center => HAlign::Center,
            NSTextAlignment::Justified => HAlign::Stretch,
            _ => HAlign::Left,
        }
    }

    pub fn set_halign(&mut self, align: HAlign) {
        unsafe {
            let align = match align {
                HAlign::Left => NSTextAlignment::Left,
                HAlign::Center => NSTextAlignment::Center,
                HAlign::Right => NSTextAlignment::Right,
                HAlign::Stretch => NSTextAlignment::Justified,
            };
            self.view.setAlignment(align);
            self.pview.setAlignment(align);
        }
    }

    pub async fn wait_change(&self) {
        self.delegate.ivars().changed.wait().await
    }
}

#[derive(Debug, Default, Clone)]
struct EditDelegateIvars {
    changed: Callback,
}

define_class! {
    #[unsafe(super(NSObject))]
    #[name = "WinioEditDelegate"]
    #[ivars = EditDelegateIvars]
    #[thread_kind = MainThreadOnly]
    #[derive(Debug)]
    struct EditDelegate;

    #[allow(non_snake_case)]
    impl EditDelegate {
        #[unsafe(method_id(init))]
        fn init(this: Allocated<Self>) -> Option<Retained<Self>> {
            let this = this.set_ivars(EditDelegateIvars::default());
            unsafe { msg_send![super(this), init] }
        }
    }

    unsafe impl NSObjectProtocol for EditDelegate {}

    #[allow(non_snake_case)]
    unsafe impl NSControlTextEditingDelegate for EditDelegate {
        #[unsafe(method(controlTextDidChange:))]
        fn controlTextDidChange(&self, _notification: &NSNotification) {
            self.ivars().changed.signal(());
        }
    }

    unsafe impl NSTextFieldDelegate for EditDelegate {}
}

impl EditDelegate {
    pub fn new(mtm: MainThreadMarker) -> Retained<Self> {
        unsafe { msg_send![mtm.alloc::<Self>(), init] }
    }
}
