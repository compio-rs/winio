use inherit_methods_macro::inherit_methods;
use objc2::{
    DefinedClass, MainThreadMarker, MainThreadOnly, define_class, msg_send,
    rc::{Allocated, Retained},
};
use objc2_app_kit::{NSControlSize, NSEvent, NSScroller, NSScrollerStyle};
use objc2_foundation::{NSPoint, NSRect, NSSize};
use winio_callback::Callback;
use winio_handle::{AsContainer, BorrowedContainer};
use winio_primitive::{Orient, Point, Size};

use crate::{GlobalRuntime, ui::Widget};

#[derive(Debug)]
struct ScrollBarImpl {
    handle: Widget,
    view: Retained<CustomScroller>,
    min: usize,
    max: usize,
}

#[inherit_methods(from = "self.handle")]
impl ScrollBarImpl {
    pub fn new(parent: impl AsContainer, vertical: bool) -> Self {
        unsafe {
            let parent = parent.as_container();
            let mtm = parent.mtm();

            let view = CustomScroller::new(
                mtm,
                if vertical {
                    NSRect::new(NSPoint::ZERO, NSSize::new(10.0, 20.0))
                } else {
                    NSRect::new(NSPoint::ZERO, NSSize::new(20.0, 10.0))
                },
            );
            let handle = Widget::from_nsview(parent, Retained::cast_unchecked(view.clone()));

            view.setEnabled(true);

            Self {
                handle,
                view,
                min: 0,
                max: 0,
            }
        }
    }

    pub fn is_visible(&self) -> bool;

    pub fn set_visible(&mut self, v: bool);

    pub fn is_enabled(&self) -> bool;

    pub fn set_enabled(&mut self, v: bool);

    pub fn loc(&self) -> Point;

    pub fn set_loc(&mut self, p: Point);

    pub fn size(&self) -> Size;

    pub fn set_size(&mut self, v: Size);

    pub fn tooltip(&self) -> String;

    pub fn set_tooltip(&mut self, s: impl AsRef<str>);

    pub fn minimum(&self) -> usize {
        self.min
    }

    pub fn set_minimum(&mut self, v: usize) {
        let pos = self.pos();
        self.min = v;
        self.set_pos(pos);
    }

    pub fn maximum(&self) -> usize {
        self.max
    }

    pub fn set_maximum(&mut self, v: usize) {
        let pos = self.pos();
        self.max = v;
        self.set_pos(pos);
    }

    pub fn page(&self) -> usize {
        (unsafe { self.view.knobProportion() } * (self.max - self.min) as f64) as usize
    }

    pub fn set_page(&mut self, v: usize) {
        unsafe {
            self.view
                .setKnobProportion(v as f64 / ((self.max - self.min) as f64));
        }
    }

    pub fn pos(&self) -> usize {
        (unsafe { self.view.doubleValue() } * (self.max - self.page() - self.min) as f64) as usize
    }

    pub fn set_pos(&mut self, v: usize) {
        unsafe {
            self.view
                .setDoubleValue(v as f64 / ((self.max - self.page() - self.min) as f64));
        }
    }

    pub async fn wait_change(&self) {
        self.view.ivars().on_move.wait().await
    }
}

winio_handle::impl_as_widget!(ScrollBarImpl, handle);

#[derive(Debug, Default)]
struct CustomScrollerIvars {
    on_move: Callback,
}

define_class! {
    #[unsafe(super(NSScroller))]
    #[name = "WinioCustomScroller"]
    #[ivars = CustomScrollerIvars]
    #[thread_kind = MainThreadOnly]
    #[derive(Debug)]
    struct CustomScroller;

    #[allow(non_snake_case)]
    impl CustomScroller {
        #[unsafe(method_id(initWithFrame:))]
        fn initWithFrame(this: Allocated<Self>, frame: NSRect) -> Option<Retained<Self>> {
            let this = this.set_ivars(CustomScrollerIvars::default());
            unsafe { msg_send![super(this), initWithFrame: frame] }
        }

        #[unsafe(method(trackKnob:))]
        unsafe fn trackKnob(&self, event: &NSEvent) {
            let () = unsafe { msg_send![super(self), trackKnob:event] };
            self.ivars().on_move.signal::<GlobalRuntime>(());
        }
    }
}

impl CustomScroller {
    pub fn new(mtm: MainThreadMarker, frame: NSRect) -> Retained<Self> {
        unsafe { msg_send![mtm.alloc::<Self>(), initWithFrame: frame] }
    }
}

#[derive(Debug)]
pub struct ScrollBar {
    handle: ScrollBarImpl,
    vertical: bool,
}

#[inherit_methods(from = "self.handle")]
impl ScrollBar {
    pub fn new(parent: impl AsContainer) -> Self {
        let handle = ScrollBarImpl::new(&parent, false);
        Self {
            handle,
            vertical: false,
        }
    }

    fn recreate(&mut self, vertical: bool) {
        let parent = self.handle.handle.parent();
        let mut new_handle =
            ScrollBarImpl::new(unsafe { BorrowedContainer::borrow_raw(parent) }, vertical);
        new_handle.set_visible(self.handle.is_visible());
        new_handle.set_enabled(self.handle.is_enabled());
        new_handle.set_loc(self.handle.loc());
        new_handle.set_size(self.handle.size());
        new_handle.set_tooltip(self.handle.tooltip());
        new_handle.set_minimum(self.handle.minimum());
        new_handle.set_maximum(self.handle.maximum());
        new_handle.set_page(self.handle.page());
        new_handle.set_pos(self.handle.pos());
        self.handle = new_handle;
    }

    pub fn is_visible(&self) -> bool;

    pub fn set_visible(&mut self, v: bool);

    pub fn is_enabled(&self) -> bool;

    pub fn set_enabled(&mut self, v: bool);

    pub fn preferred_size(&self) -> Size {
        let width = unsafe {
            NSScroller::scrollerWidthForControlSize_scrollerStyle(
                NSControlSize::Regular,
                NSScrollerStyle::Overlay,
                self.handle.view.mtm(),
            )
        };
        if self.vertical {
            Size::new(width, 0.0)
        } else {
            Size::new(0.0, width)
        }
    }

    pub fn loc(&self) -> Point;

    pub fn set_loc(&mut self, p: Point);

    pub fn size(&self) -> Size;

    pub fn set_size(&mut self, v: Size);

    pub fn tooltip(&self) -> String;

    pub fn set_tooltip(&mut self, s: impl AsRef<str>);

    pub fn orient(&self) -> Orient {
        if self.vertical {
            Orient::Vertical
        } else {
            Orient::Horizontal
        }
    }

    pub fn set_orient(&mut self, v: Orient) {
        let v = matches!(v, Orient::Vertical);
        if self.vertical != v {
            self.recreate(v);
            self.vertical = v;
        }
    }

    pub fn minimum(&self) -> usize;

    pub fn set_minimum(&mut self, v: usize);

    pub fn maximum(&self) -> usize;

    pub fn set_maximum(&mut self, v: usize);

    pub fn page(&self) -> usize;

    pub fn set_page(&mut self, v: usize);

    pub fn pos(&self) -> usize;

    pub fn set_pos(&mut self, v: usize);

    pub async fn wait_change(&self) {
        self.handle.wait_change().await
    }
}

winio_handle::impl_as_widget!(ScrollBar, handle);
