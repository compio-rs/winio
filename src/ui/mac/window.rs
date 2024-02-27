use std::{io, rc::Rc};

use icrate::{
    objc2::{
        declare_class, msg_send_id,
        mutability::MainThreadOnly,
        rc::{Allocated, Id},
        runtime::ProtocolObject,
        ClassType, DeclaredClass,
    },
    AppKit::{
        NSBackingStoreBuffered, NSWindow, NSWindowDelegate, NSWindowStyleMaskClosable,
        NSWindowStyleMaskMiniaturizable, NSWindowStyleMaskResizable, NSWindowStyleMaskTitled,
    },
    Foundation::{CGPoint, CGSize, MainThreadMarker, NSObject, NSObjectProtocol, NSRect},
};

use super::callback::Callback;

pub struct Window {
    wnd: Id<NSWindow>,
    delegate: Id<WindowDelegate>,
}

impl Window {
    pub fn new() -> io::Result<Rc<Self>> {
        unsafe {
            let mtm = MainThreadMarker::new().unwrap();

            let frame = NSRect::new(CGPoint::ZERO, CGSize::new(100.0, 100.0));
            let wnd = {
                NSWindow::initWithContentRect_styleMask_backing_defer(
                    mtm.alloc(),
                    frame,
                    NSWindowStyleMaskTitled
                        | NSWindowStyleMaskClosable
                        | NSWindowStyleMaskResizable
                        | NSWindowStyleMaskMiniaturizable,
                    NSBackingStoreBuffered,
                    false,
                )
            };

            let delegate = WindowDelegate::new(mtm);
            let del_obj = ProtocolObject::from_id(delegate.clone());
            wnd.setDelegate(Some(&del_obj));

            wnd.setIsVisible(true);
            Ok(Rc::new(Self { wnd, delegate }))
        }
    }

    pub async fn wait_close(&self) {
        self.delegate.ivars().should_close.wait().await
    }
}

#[derive(Clone)]
struct WindowDelegateIvars {
    should_close: Callback,
}

declare_class! {
    struct WindowDelegate;

    unsafe impl ClassType for WindowDelegate {
        type Super = NSObject;
        type Mutability = MainThreadOnly;
        const NAME: &'static str = "WinioWindowDelegate";
    }

    impl DeclaredClass for WindowDelegate {
        type Ivars = WindowDelegateIvars;
    }

    unsafe impl WindowDelegate {
        #[method_id(init)]
        fn init(this: Allocated<Self>) -> Option<Id<Self>> {
            let this = this.set_ivars(WindowDelegateIvars {
                should_close: Callback::new(),
            });
            unsafe { msg_send_id![super(this), init] }
        }
    }

    unsafe impl NSObjectProtocol for WindowDelegate {}

    #[allow(non_snake_case)]
    unsafe impl NSWindowDelegate for WindowDelegate {
        #[method(windowShouldClose:)]
        unsafe fn windowShouldClose(&self, _sender: &NSWindow) -> bool {
            self.ivars().should_close.signal()
        }
    }
}

impl WindowDelegate {
    pub fn new(mtm: MainThreadMarker) -> Id<Self> {
        unsafe { msg_send_id![mtm.alloc::<Self>(), init] }
    }
}
