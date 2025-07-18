use inherit_methods_macro::inherit_methods;
use objc2::{
    DefinedClass, MainThreadMarker, MainThreadOnly, define_class, msg_send,
    rc::{Allocated, Retained},
};
use objc2_app_kit::{NSEvent, NSScroller};
use objc2_foundation::{NSPoint, NSRect, NSSize};
use winio_callback::Callback;
use winio_handle::{AsRawWidget, AsWidget, AsWindow, BorrowedWidget, RawWidget};
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
    pub fn new(parent: impl AsWindow, vertical: bool) -> Self {
        unsafe {
            let mtm = MainThreadMarker::new().unwrap();

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

    pub fn range(&self) -> (usize, usize) {
        (self.min, self.max)
    }

    pub fn set_range(&mut self, min: usize, max: usize) {
        self.min = min;
        self.max = max;
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

impl AsRawWidget for ScrollBarImpl {
    fn as_raw_widget(&self) -> RawWidget {
        self.handle.as_raw_widget()
    }
}

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
    vhandle: ScrollBarImpl,
    vertical: bool,
}

impl ScrollBar {
    pub fn new(parent: impl AsWindow) -> Self {
        let parent = parent.as_window();
        let handle = ScrollBarImpl::new(&parent, false);
        let mut vhandle = ScrollBarImpl::new(&parent, true);
        vhandle.set_visible(false);
        Self {
            handle,
            vhandle,
            vertical: false,
        }
    }

    pub fn is_visible(&self) -> bool {
        if self.vertical {
            &self.vhandle
        } else {
            &self.handle
        }
        .is_visible()
    }

    pub fn set_visible(&mut self, v: bool) {
        if self.vertical {
            &mut self.vhandle
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
        self.vhandle.set_enabled(v);
    }

    pub fn preferred_size(&self) -> Size {
        if self.vertical {
            Size::new(20.0, 0.0)
        } else {
            Size::new(0.0, 20.0)
        }
    }

    pub fn loc(&self) -> Point {
        self.handle.loc()
    }

    pub fn set_loc(&mut self, p: Point) {
        self.handle.set_loc(p);
        self.vhandle.set_loc(p);
    }

    pub fn size(&self) -> Size {
        self.handle.size()
    }

    pub fn set_size(&mut self, v: Size) {
        self.handle.set_size(v);
        self.vhandle.set_size(v);
    }

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
            if v {
                self.vhandle.set_pos(self.handle.pos());
                self.vhandle.set_visible(self.handle.is_visible());
                self.handle.set_visible(false);
            } else {
                self.handle.set_pos(self.vhandle.pos());
                self.handle.set_visible(self.vhandle.is_visible());
                self.vhandle.set_visible(false);
            }
            self.vertical = v;
        }
    }

    pub fn range(&self) -> (usize, usize) {
        self.handle.range()
    }

    pub fn set_range(&mut self, min: usize, max: usize) {
        self.handle.set_range(min, max);
        self.vhandle.set_range(min, max);
    }

    pub fn page(&self) -> usize {
        self.handle.page()
    }

    pub fn set_page(&mut self, v: usize) {
        self.handle.set_page(v);
        self.vhandle.set_page(v);
    }

    pub fn pos(&self) -> usize {
        if self.vertical {
            &self.vhandle
        } else {
            &self.handle
        }
        .pos()
    }

    pub fn set_pos(&mut self, v: usize) {
        self.handle.set_pos(v);
        self.vhandle.set_pos(v);
    }

    pub async fn wait_change(&self) {
        if self.vertical {
            &self.vhandle
        } else {
            &self.handle
        }
        .wait_change()
        .await
    }
}

impl AsRawWidget for ScrollBar {
    fn as_raw_widget(&self) -> RawWidget {
        if self.vertical {
            &self.vhandle
        } else {
            &self.handle
        }
        .as_raw_widget()
    }
}

impl AsWidget for ScrollBar {
    fn as_widget(&self) -> BorrowedWidget<'_> {
        unsafe { BorrowedWidget::borrow_raw(self.as_raw_widget()) }
    }
}
