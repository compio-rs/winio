use std::fmt::Debug;

use inherit_methods_macro::inherit_methods;
use objc2::{
    DeclaredClass, MainThreadOnly, define_class, msg_send,
    rc::{Allocated, Retained, Weak},
    runtime::ProtocolObject,
};
use objc2_app_kit::{
    NSBackingStoreType, NSControl, NSScreen, NSView, NSWindow, NSWindowDelegate, NSWindowStyleMask,
};
use objc2_foundation::{
    MainThreadMarker, NSNotification, NSObject, NSObjectProtocol, NSPoint, NSRect, NSSize, NSString,
};
use winio_callback::Callback;
use winio_handle::{
    AsContainer, AsRawContainer, AsRawWidget, AsRawWindow, AsWindow, BorrowedContainer,
    BorrowedWindow, RawContainer, RawWidget, RawWindow,
};
use winio_primitive::{Point, Rect, Size};

use crate::{
    GlobalRuntime,
    ui::{from_cgsize, from_nsstring, to_cgsize, transform_cgrect, transform_rect},
};

#[derive(Debug)]
pub struct Window {
    wnd: Retained<NSWindow>,
    delegate: Retained<WindowDelegate>,
}

impl Window {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        unsafe {
            let mtm = MainThreadMarker::new().unwrap();

            let frame = NSRect::new(NSPoint::ZERO, NSSize::new(100.0, 100.0));
            let wnd = {
                NSWindow::initWithContentRect_styleMask_backing_defer(
                    mtm.alloc(),
                    frame,
                    NSWindowStyleMask::Titled
                        | NSWindowStyleMask::Closable
                        | NSWindowStyleMask::Resizable
                        | NSWindowStyleMask::Miniaturizable,
                    NSBackingStoreType::Buffered,
                    false,
                )
            };

            let delegate = WindowDelegate::new(mtm);
            let del_obj = ProtocolObject::from_ref(&*delegate);
            wnd.setDelegate(Some(del_obj));
            wnd.setAcceptsMouseMovedEvents(true);
            wnd.makeKeyWindow();
            let mut this = Self { wnd, delegate };
            this.set_loc(Point::zero());
            this
        }
    }

    fn screen(&self) -> Retained<NSScreen> {
        self.wnd.screen().unwrap()
    }

    pub fn is_visible(&self) -> bool {
        self.wnd.isVisible()
    }

    pub fn set_visible(&mut self, v: bool) {
        unsafe { self.wnd.setIsVisible(v) }
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
        let ydiff = v.height - frame.size.height;
        frame.size = to_cgsize(v);
        frame.origin.y -= ydiff;
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

impl AsWindow for Window {
    fn as_window(&self) -> BorrowedWindow<'_> {
        unsafe { BorrowedWindow::borrow_raw(self.as_raw_window()) }
    }
}

impl AsRawContainer for Window {
    fn as_raw_container(&self) -> RawContainer {
        self.wnd.contentView().unwrap()
    }
}

impl AsContainer for Window {
    fn as_container(&self) -> BorrowedContainer<'_> {
        unsafe { BorrowedContainer::borrow_raw(self.as_raw_container()) }
    }
}

#[derive(Debug, Default)]
struct WindowDelegateIvars {
    did_resize: Callback,
    did_move: Callback,
    should_close: Callback,
}

define_class! {
    #[unsafe(super(NSObject))]
    #[name = "WinioWindowDelegate"]
    #[ivars = WindowDelegateIvars]
    #[thread_kind = MainThreadOnly]
    #[derive(Debug)]
    struct WindowDelegate;

    impl WindowDelegate {
        #[unsafe(method_id(init))]
        fn init(this: Allocated<Self>) -> Option<Retained<Self>> {
            let this = this.set_ivars(WindowDelegateIvars::default());
            unsafe { msg_send![super(this), init] }
        }
    }

    unsafe impl NSObjectProtocol for WindowDelegate {}

    #[allow(non_snake_case)]
    unsafe impl NSWindowDelegate for WindowDelegate {
        #[unsafe(method(windowDidResize:))]
        unsafe fn windowDidResize(&self, _notification: &NSNotification) {
            self.ivars().did_resize.signal::<GlobalRuntime>(());
        }

        #[unsafe(method(windowDidMove:))]
        unsafe fn windowDidMove(&self, _notification: &NSNotification) {
            self.ivars().did_move.signal::<GlobalRuntime>(());
        }

        #[unsafe(method(windowShouldClose:))]
        unsafe fn windowShouldClose(&self, _sender: &NSWindow) -> bool {
            self.ivars().should_close.signal::<GlobalRuntime>(())
        }
    }
}

impl WindowDelegate {
    pub fn new(mtm: MainThreadMarker) -> Retained<Self> {
        unsafe { msg_send![mtm.alloc::<Self>(), init] }
    }
}

#[derive(Debug)]
pub(crate) struct Widget {
    parent: Weak<NSView>,
    view: Retained<NSView>,
}

impl Widget {
    pub fn from_nsview(parent: impl AsContainer, view: Retained<NSView>) -> Self {
        unsafe {
            let parent = parent.as_container().as_raw_container();
            parent.addSubview(&view);
            let mut this = Self {
                parent: Weak::from_retained(&parent),
                view,
            };
            this.set_loc(Point::zero());
            this
        }
    }

    pub fn parent(&self) -> Retained<NSView> {
        self.parent.load().unwrap()
    }

    pub fn is_visible(&self) -> bool {
        unsafe { !self.view.isHidden() }
    }

    pub fn set_visible(&mut self, v: bool) {
        self.view.setHidden(!v)
    }

    pub fn is_enabled(&self) -> bool {
        unsafe { Retained::cast_unchecked::<NSControl>(self.view.clone()).isEnabled() }
    }

    pub fn set_enabled(&mut self, v: bool) {
        unsafe {
            Retained::cast_unchecked::<NSControl>(self.view.clone()).setEnabled(v);
        }
    }

    pub fn preferred_size(&self) -> Size {
        unsafe {
            from_cgsize(
                Retained::cast_unchecked::<NSControl>(self.view.clone()).sizeThatFits(NSSize::ZERO),
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
        let ydiff = v.height - frame.size.height;
        frame.size = to_cgsize(v);
        frame.origin.y -= ydiff;
        unsafe {
            self.view.setFrame(frame);
        }
    }
}

impl Drop for Widget {
    fn drop(&mut self) {
        unsafe {
            self.view.removeFromSuperview();
        }
    }
}

impl AsRawWidget for Widget {
    fn as_raw_widget(&self) -> RawWidget {
        self.view.clone()
    }
}

impl AsRawContainer for Widget {
    fn as_raw_container(&self) -> RawContainer {
        self.view.clone()
    }
}

#[derive(Debug)]
pub struct View {
    handle: Widget,
}

#[inherit_methods(from = "self.handle")]
impl View {
    pub fn new(parent: impl AsContainer) -> Self {
        unsafe {
            let mtm = MainThreadMarker::new().unwrap();

            let view = NSView::new(mtm);
            let handle = Widget::from_nsview(parent, Retained::cast_unchecked(view.clone()));

            Self { handle }
        }
    }

    pub fn is_visible(&self) -> bool;

    pub fn set_visible(&mut self, v: bool);

    pub fn loc(&self) -> Point;

    pub fn set_loc(&mut self, p: Point);

    pub fn size(&self) -> Size;

    pub fn set_size(&mut self, v: Size);
}

winio_handle::impl_as_widget!(View, handle);
winio_handle::impl_as_container!(View, handle);
