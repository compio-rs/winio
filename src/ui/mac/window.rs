use std::{io, rc::Rc};

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
    CGPoint, CGSize, MainThreadMarker, NSNotification, NSObject, NSObjectProtocol, NSRect,
    NSString, NSUserDefaults,
};

use super::{from_cgsize, from_nsstring, to_cgsize};
use crate::{Callback, ColorTheme, Point, Size};

pub trait AsNSView {
    fn as_nsview(&self) -> Id<NSView>;
}

impl<T: AsNSView> AsNSView for &'_ T {
    fn as_nsview(&self) -> Id<NSView> {
        (**self).as_nsview()
    }
}

impl<T: AsNSView> AsNSView for Rc<T> {
    fn as_nsview(&self) -> Id<NSView> {
        (**self).as_nsview()
    }
}

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
            Ok(Rc::new(Self { wnd, delegate }))
        }
    }

    pub fn as_nswindow(&self) -> Id<NSWindow> {
        self.wnd.clone()
    }

    fn screen(&self) -> io::Result<Id<NSScreen>> {
        self.wnd
            .screen()
            .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "cannot get screen of the window"))
    }

    pub fn color_theme(&self) -> ColorTheme {
        unsafe {
            let osx_mode = NSUserDefaults::standardUserDefaults()
                .stringForKey(&NSString::from_str("AppleInterfaceStyle"));
            let is_dark = osx_mode
                .map(|mode| mode.isEqualToString(&NSString::from_str("Dark")))
                .unwrap_or_default();
            if is_dark {
                ColorTheme::Dark
            } else {
                ColorTheme::Light
            }
        }
    }

    pub fn loc(&self) -> io::Result<Point> {
        let frame = self.wnd.frame();
        let screen_frame = self.screen()?.frame();
        Ok(Point::new(
            frame.origin.x,
            screen_frame.size.height - frame.size.height - frame.origin.y,
        ))
    }

    pub fn set_loc(&self, p: Point) -> io::Result<()> {
        let mut frame = self.wnd.frame();
        let screen_frame = self.screen()?.frame();
        frame.origin.x = p.x;
        frame.origin.y = screen_frame.size.height - frame.size.height - p.y;
        self.wnd.setFrame_display(frame, true);
        Ok(())
    }

    pub fn size(&self) -> io::Result<Size> {
        Ok(from_cgsize(self.wnd.frame().size))
    }

    pub fn set_size(&self, v: Size) -> io::Result<()> {
        let mut frame = self.wnd.frame();
        frame.size = to_cgsize(v);
        self.wnd.setFrame_display(frame, true);
        Ok(())
    }

    pub fn client_size(&self) -> io::Result<Size> {
        Ok(from_cgsize(self.as_nsview().frame().size))
    }

    pub fn text(&self) -> io::Result<String> {
        Ok(from_nsstring(&self.wnd.title()))
    }

    pub fn set_text(&self, s: impl AsRef<str>) -> io::Result<()> {
        self.wnd.setTitle(&NSString::from_str(s.as_ref()));
        Ok(())
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

impl AsNSView for Window {
    fn as_nsview(&self) -> Id<NSView> {
        self.wnd
            .contentView()
            .expect("a window should contain a view")
    }
}

#[derive(Default, Clone)]
struct WindowDelegateIvars {
    did_resize: Callback,
    did_move: Callback,
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
    pub fn from_nsview(parent: Id<NSView>, view: Id<NSView>) -> Self {
        unsafe {
            parent.addSubview(&view);
            if view.is_kind_of::<NSControl>() {
                Id::cast::<NSControl>(view.clone()).sizeToFit();
            }
            Self {
                parent: WeakId::from_id(&parent),
                view,
            }
        }
    }

    pub fn parent(&self) -> io::Result<Id<NSView>> {
        self.parent
            .load()
            .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "cannot find parent view"))
    }

    pub fn loc(&self) -> io::Result<Point> {
        let frame = self.view.frame();
        let screen_frame = self.parent()?.frame();
        Ok(Point::new(
            frame.origin.x,
            screen_frame.size.height - frame.size.height - frame.origin.y,
        ))
    }

    pub fn set_loc(&self, p: Point) -> io::Result<()> {
        let mut frame = self.view.frame();
        let screen_frame = self.parent()?.frame();
        frame.origin.x = p.x;
        frame.origin.y = screen_frame.size.height - frame.size.height - p.y;
        unsafe {
            self.view.setFrame(frame);
        }
        Ok(())
    }

    pub fn size(&self) -> io::Result<Size> {
        Ok(from_cgsize(self.view.frame().size))
    }

    pub fn set_size(&self, v: Size) -> io::Result<()> {
        let mut frame = self.view.frame();
        frame.size = to_cgsize(v);
        unsafe {
            self.view.setFrame(frame);
        }
        Ok(())
    }
}

impl AsNSView for Widget {
    fn as_nsview(&self) -> Id<NSView> {
        self.view.clone()
    }
}
