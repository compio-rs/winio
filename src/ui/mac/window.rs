use std::{io, rc::Rc};

use icrate::{
    objc2::rc::Id,
    AppKit::{
        NSBackingStoreBuffered, NSWindow, NSWindowStyleMaskClosable,
        NSWindowStyleMaskMiniaturizable, NSWindowStyleMaskResizable, NSWindowStyleMaskTitled,
    },
    Foundation::{CGPoint, CGSize, MainThreadMarker, NSRect},
};

pub struct Window {
    wnd: Id<NSWindow>,
}

impl Window {
    pub fn new() -> io::Result<Rc<Self>> {
        unsafe {
            let frame = NSRect::new(CGPoint::ZERO, CGSize::new(100.0, 100.0));
            let wnd = {
                NSWindow::initWithContentRect_styleMask_backing_defer(
                    MainThreadMarker::new().unwrap().alloc(),
                    frame,
                    NSWindowStyleMaskTitled
                        | NSWindowStyleMaskClosable
                        | NSWindowStyleMaskResizable
                        | NSWindowStyleMaskMiniaturizable,
                    NSBackingStoreBuffered,
                    false,
                )
            };
            wnd.setIsVisible(true);
            Ok(Rc::new(Self { wnd }))
        }
    }
}
