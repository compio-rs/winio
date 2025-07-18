use inherit_methods_macro::inherit_methods;
use objc2::{
    DefinedClass, MainThreadMarker, MainThreadOnly, define_class, msg_send,
    rc::{Allocated, Retained},
};
use objc2_app_kit::{NSEvent, NSScroller};
use winio_callback::Callback;
use winio_handle::AsWindow;
use winio_primitive::{Orient, Point, Size};

use crate::{GlobalRuntime, ui::Widget};

#[derive(Debug)]
pub struct ScrollBar {
    handle: Widget,
    view: Retained<CustomScroller>,
    min: usize,
    max: usize,
}

#[inherit_methods(from = "self.handle")]
impl ScrollBar {
    pub fn new(parent: impl AsWindow) -> Self {
        unsafe {
            let mtm = MainThreadMarker::new().unwrap();

            let view = CustomScroller::new(mtm);
            let handle = Widget::from_nsview(parent, Retained::cast_unchecked(view.clone()));

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

    pub fn preferred_size(&self) -> Size;

    pub fn loc(&self) -> Point;

    pub fn set_loc(&mut self, p: Point);

    pub fn size(&self) -> Size;

    pub fn set_size(&mut self, v: Size);

    pub fn orient(&self) -> Orient {
        let size = self.size();
        if size.width >= size.height {
            Orient::Horizontal
        } else {
            Orient::Vertical
        }
    }

    pub fn set_orient(&mut self, v: Orient) {
        if self.orient() != v {
            let mut size = self.size();
            (size.width, size.height) = (size.height, size.width);
            self.set_size(size);
        }
    }

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
        (unsafe { self.view.doubleValue() } * (self.max - self.min) as f64) as usize
    }

    pub fn set_pos(&mut self, v: usize) {
        unsafe {
            self.view
                .setDoubleValue(v as f64 / ((self.max - self.min) as f64));
        }
    }

    pub async fn wait_change(&self) {
        self.view.ivars().on_move.wait().await
    }
}

winio_handle::impl_as_widget!(ScrollBar, handle);

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
        #[unsafe(method_id(init))]
        fn init(this: Allocated<Self>) -> Option<Retained<Self>> {
            let this = this.set_ivars(CustomScrollerIvars::default());
            unsafe { msg_send![super(this), init] }
        }

        #[unsafe(method(trackKnob:))]
        unsafe fn trackKnob(&self, event: &NSEvent) {
            let () = unsafe { msg_send![super(self), trackKnob:event] };
            self.ivars().on_move.signal::<GlobalRuntime>(());
        }
    }
}

impl CustomScroller {
    pub fn new(mtm: MainThreadMarker) -> Retained<Self> {
        unsafe { msg_send![mtm.alloc::<Self>(), init] }
    }
}
