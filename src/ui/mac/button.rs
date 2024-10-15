use objc2::{
    ClassType, DeclaredClass, declare_class, msg_send_id,
    mutability::MainThreadOnly,
    rc::{Allocated, Id},
    sel,
};
use objc2_app_kit::{NSBezelStyle, NSButton};
use objc2_foundation::{MainThreadMarker, NSObject, NSString};

use crate::{
    AsRawWindow, AsWindow, Point, Size,
    ui::{Callback, Widget, from_nsstring},
};

#[derive(Debug)]
pub struct Button {
    handle: Widget,
    view: Id<NSButton>,
    delegate: Id<ButtonDelegate>,
}

impl Button {
    pub fn new(parent: impl AsWindow) -> Self {
        unsafe {
            let mtm = MainThreadMarker::new().unwrap();

            let view = NSButton::new(mtm);
            let handle =
                Widget::from_nsview(parent.as_window().as_raw_window(), Id::cast(view.clone()));

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

#[derive(Default, Clone)]
struct ButtonDelegateIvars {
    action: Callback,
}

declare_class! {
    #[derive(Debug)]
    struct ButtonDelegate;

    unsafe impl ClassType for ButtonDelegate {
        type Super = NSObject;
        type Mutability = MainThreadOnly;
        const NAME: &'static str = "WinioButtonDelegate";
    }

    impl DeclaredClass for ButtonDelegate {
        type Ivars = ButtonDelegateIvars;
    }

    #[allow(non_snake_case)]
    unsafe impl ButtonDelegate {
        #[method_id(init)]
        fn init(this: Allocated<Self>) -> Option<Id<Self>> {
            let this = this.set_ivars(ButtonDelegateIvars::default());
            unsafe { msg_send_id![super(this), init] }
        }

        #[method(onAction)]
        unsafe fn onAction(&self) {
            self.ivars().action.signal(());
        }
    }
}

impl ButtonDelegate {
    pub fn new(mtm: MainThreadMarker) -> Id<Self> {
        unsafe { msg_send_id![mtm.alloc::<Self>(), init] }
    }
}
