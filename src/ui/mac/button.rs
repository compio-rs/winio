use std::{io, rc::Rc};

use icrate::{
    objc2::{
        declare_class, msg_send_id,
        mutability::MainThreadOnly,
        rc::{Allocated, Id},
        sel, ClassType, DeclaredClass,
    },
    AppKit::{NSBezelStyleFlexiblePush, NSButton},
    Foundation::{MainThreadMarker, NSObject, NSString},
};

use super::{callback::Callback, from_nsstring};
use crate::{AsNSView, Point, Size, Widget};

#[derive(Debug)]
pub struct Button {
    handle: Widget,
    view: Id<NSButton>,
    delegate: Id<ButtonDelegate>,
}

impl Button {
    pub fn new(parent: impl AsNSView) -> io::Result<Rc<Self>> {
        unsafe {
            let mtm = MainThreadMarker::new().unwrap();

            let view = NSButton::new(mtm);
            let handle = Widget::from_nsview(parent.as_nsview(), Id::cast(view.clone()));

            let delegate = ButtonDelegate::new(mtm);
            view.setTarget(Some(&delegate));
            view.setAction(Some(sel!(onAction)));

            view.setBezelStyle(NSBezelStyleFlexiblePush);
            Ok(Rc::new(Self {
                handle,
                view,
                delegate,
            }))
        }
    }

    pub fn loc(&self) -> io::Result<Point> {
        self.handle.loc()
    }

    pub fn set_loc(&self, p: Point) -> io::Result<()> {
        self.handle.set_loc(p)
    }

    pub fn size(&self) -> io::Result<Size> {
        self.handle.size()
    }

    pub fn set_size(&self, v: Size) -> io::Result<()> {
        self.handle.set_size(v)
    }

    pub fn text(&self) -> io::Result<String> {
        unsafe { Ok(from_nsstring(&self.view.title())) }
    }

    pub fn set_text(&self, s: impl AsRef<str>) -> io::Result<()> {
        unsafe {
            self.view.setTitle(&NSString::from_str(s.as_ref()));
            self.view.sizeToFit();
        }
        Ok(())
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
            self.ivars().action.signal();
        }
    }
}

impl ButtonDelegate {
    pub fn new(mtm: MainThreadMarker) -> Id<Self> {
        unsafe { msg_send_id![mtm.alloc::<Self>(), init] }
    }
}
