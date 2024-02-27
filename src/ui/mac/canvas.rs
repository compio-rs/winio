use std::{io, rc::Rc};

use icrate::{
    objc2::{
        declare_class, msg_send_id,
        mutability::MainThreadOnly,
        rc::{Allocated, Id},
        ClassType, DeclaredClass,
    },
    AppKit::NSView,
    Foundation::{MainThreadMarker, NSRect},
};

use super::callback::Callback;
use crate::{AsNSView, Point, Size, Widget};

#[derive(Debug)]
pub struct Canvas {
    view: Id<CanvasView>,
    handle: Widget,
}

impl Canvas {
    pub fn new(parent: impl AsNSView) -> io::Result<Rc<Self>> {
        let view = CanvasView::new(MainThreadMarker::new().unwrap());
        let handle = Widget::from_nsview(parent.as_nsview(), Id::into_super(view.clone()));
        Ok(Rc::new(Self { view, handle }))
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

    pub fn redraw(&self) -> io::Result<()> {
        unsafe {
            self.handle.as_nsview().setNeedsDisplay(true);
        }
        Ok(())
    }

    pub async fn wait_redraw(&self) {
        self.view.ivars().draw_rect.wait().await
    }
}

impl AsNSView for Canvas {
    fn as_nsview(&self) -> Id<NSView> {
        self.handle.as_nsview()
    }
}

#[derive(Default, Clone)]
struct CanvasViewIvars {
    draw_rect: Callback,
}

declare_class! {
    #[derive(Debug)]
    struct CanvasView;

    unsafe impl ClassType for CanvasView {
        type Super = NSView;
        type Mutability = MainThreadOnly;
        const NAME: &'static str = "WinioCanvasView";
    }

    impl DeclaredClass for CanvasView {
        type Ivars = CanvasViewIvars;
    }

    #[allow(non_snake_case)]
    unsafe impl CanvasView {
        #[method_id(init)]
        fn init(this: Allocated<Self>) -> Option<Id<Self>> {
            let this = this.set_ivars(CanvasViewIvars::default());
            unsafe { msg_send_id![super(this), init] }
        }

        #[method(drawRect:)]
        unsafe fn drawRect(&self, _dirty_rect: NSRect) {
            self.ivars().draw_rect.signal();
        }
    }
}

impl CanvasView {
    pub fn new(mtm: MainThreadMarker) -> Id<Self> {
        unsafe { msg_send_id![mtm.alloc::<Self>(), init] }
    }
}

pub trait Brush {}

pub trait Pen {}
