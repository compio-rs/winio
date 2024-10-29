use std::fmt::Debug;

use objc2::{
    ClassType, DeclaredClass, declare_class, msg_send_id,
    mutability::MainThreadOnly,
    rc::{Allocated, Id, WeakId},
    runtime::ProtocolObject,
};
use objc2_app_kit::{
    NSBackingStoreType, NSControl, NSScreen, NSView, NSWindow, NSWindowDelegate, NSWindowStyleMask,
};
use objc2_foundation::{
    CGPoint, CGSize, MainThreadMarker, NSNotification, NSObject, NSObjectProtocol, NSRect, NSSize,
    NSString,
};

use super::{transform_cgrect, transform_rect};
use crate::{
    AsRawWindow, AsWindow, Point, RawWindow, Rect, Size,
    ui::{Callback, from_cgsize, from_nsstring, to_cgsize},
};

#[derive(Debug)]
pub struct Window {
    wnd: Id<NSWindow>,
    delegate: Id<WindowDelegate>,
}

impl Window {
    pub fn new() -> Self {
        unsafe {
            let mtm = MainThreadMarker::new().unwrap();

            let frame = NSRect::new(CGPoint::ZERO, CGSize::new(100.0, 100.0));
            let wnd = {
                NSWindow::initWithContentRect_styleMask_backing_defer(
                    mtm.alloc(),
                    frame,
                    NSWindowStyleMask::Titled
                        | NSWindowStyleMask::Closable
                        | NSWindowStyleMask::Resizable
                        | NSWindowStyleMask::Miniaturizable,
                    NSBackingStoreType::NSBackingStoreBuffered,
                    false,
                )
            };

            let delegate = WindowDelegate::new(mtm);
            let del_obj = ProtocolObject::from_id(delegate.clone());
            wnd.setDelegate(Some(&del_obj));

            wnd.setIsVisible(true);
            Self { wnd, delegate }
        }
    }

    fn screen(&self) -> Id<NSScreen> {
        self.wnd.screen().unwrap()
    }

    pub fn loc(&self) -> Point {
        let frame = self.wnd.frame();
        let screen_frame = self.screen().frame();
        transform_cgrect(from_cgsize(screen_frame.size), frame).origin
    }

    pub fn set_loc(&mut self, p: Point) {
        let frame = self.wnd.frame();
        let screen_frame = self.screen().frame();
        let frame = transform_rect(
            from_cgsize(screen_frame.size),
            Rect::new(p, from_cgsize(frame.size)),
        );
        self.wnd.setFrame_display(frame, true);
    }

    pub fn size(&self) -> Size {
        from_cgsize(self.wnd.frame().size)
    }

    pub fn set_size(&mut self, v: Size) {
        let mut frame = self.wnd.frame();
        frame.size = to_cgsize(v);
        self.wnd.setFrame_display(frame, true);
    }

    pub fn client_size(&self) -> Size {
        from_cgsize(self.wnd.contentView().unwrap().frame().size)
    }

    pub fn text(&self) -> String {
        from_nsstring(&self.wnd.title())
    }

    pub fn set_text(&mut self, s: impl AsRef<str>) {
        self.wnd.setTitle(&NSString::from_str(s.as_ref()));
    }

    pub async fn wait_size(&self) {
        self.delegate.ivars().did_resize.wait().await
    }

    pub async fn wait_move(&self) {
        self.delegate.ivars().did_move.wait().await
    }

    pub async fn wait_close(&self) {
        self.delegate.ivars().should_close.wait().await
    }
}

impl AsRawWindow for Window {
    fn as_raw_window(&self) -> RawWindow {
        self.wnd.clone()
    }
}

#[derive(Default, Clone)]
struct WindowDelegateIvars {
    did_resize: Callback,
    did_move: Callback,
    should_close: Callback,
}

declare_class! {
    #[derive(Debug)]
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
            let this = this.set_ivars(WindowDelegateIvars::default());
            unsafe { msg_send_id![super(this), init] }
        }
    }

    unsafe impl NSObjectProtocol for WindowDelegate {}

    #[allow(non_snake_case)]
    unsafe impl NSWindowDelegate for WindowDelegate {
        #[method(windowDidResize:)]
        unsafe fn windowDidResize(&self, _notification: &NSNotification) {
            self.ivars().did_resize.signal(());
        }

        #[method(windowDidMove:)]
        unsafe fn windowDidMove(&self, _notification: &NSNotification) {
            self.ivars().did_move.signal(());
        }

        #[method(windowShouldClose:)]
        unsafe fn windowShouldClose(&self, _sender: &NSWindow) -> bool {
            self.ivars().should_close.signal(())
        }
    }
}

impl WindowDelegate {
    pub fn new(mtm: MainThreadMarker) -> Id<Self> {
        unsafe { msg_send_id![mtm.alloc::<Self>(), init] }
    }
}

#[derive(Debug)]
pub(crate) struct Widget {
    parent: WeakId<NSView>,
    view: Id<NSView>,
}

impl Widget {
    pub fn from_nsview(parent: impl AsWindow, view: Id<NSView>) -> Self {
        unsafe {
            let parent = parent.as_window().as_raw_window().contentView().unwrap();
            parent.addSubview(&view);
            Self {
                parent: WeakId::from_id(&parent),
                view,
            }
        }
    }

    pub fn parent(&self) -> Id<NSView> {
        self.parent.load().unwrap()
    }

    pub fn preferred_size(&self) -> Size {
        unsafe {
            from_cgsize(
                Id::cast::<NSControl>(self.view.clone())
                    .sizeThatFits(NSSize::new(f64::MAX, f64::MAX)),
            )
        }
    }

    pub fn loc(&self) -> Point {
        let frame = self.view.frame();
        let screen_frame = self.parent().frame();
        transform_cgrect(from_cgsize(screen_frame.size), frame).origin
    }

    pub fn set_loc(&mut self, p: Point) {
        let frame = self.view.frame();
        let screen_frame = self.parent().frame();
        let frame = transform_rect(
            from_cgsize(screen_frame.size),
            Rect::new(p, from_cgsize(frame.size)),
        );
        unsafe {
            self.view.setFrame(frame);
        }
    }

    pub fn size(&self) -> Size {
        from_cgsize(self.view.frame().size)
    }

    pub fn set_size(&mut self, v: Size) {
        let mut frame = self.view.frame();
        frame.size = to_cgsize(v);
        unsafe {
            self.view.setFrame(frame);
        }
    }
}
