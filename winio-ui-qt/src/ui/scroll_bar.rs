use cxx::{ExternType, type_id};
use inherit_methods_macro::inherit_methods;
use winio_callback::Callback;
use winio_handle::AsWindow;
use winio_primitive::{Orient, Point, Size};

use crate::{GlobalRuntime, Widget, impl_static_cast};

#[derive(Debug)]
pub struct ScrollBar {
    on_moved: Box<Callback>,
    widget: Widget<ffi::QScrollBar>,
}

#[inherit_methods(from = "self.widget")]
impl ScrollBar {
    pub fn new(parent: impl AsWindow) -> Self {
        let mut widget = unsafe { ffi::new_scroll_bar(parent.as_window().as_qt()) };
        let on_moved = Box::new(Callback::new());
        unsafe {
            ffi::scroll_bar_connect_moved(
                widget.pin_mut(),
                Self::on_moved,
                on_moved.as_ref() as *const _ as _,
            );
        }
        let mut widget = Widget::new(widget);
        widget.set_visible(true);
        Self { on_moved, widget }
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
        match self.widget.as_ref().orientation() {
            QtOrientation::Horizontal => Orient::Horizontal,
            QtOrientation::Vertical => Orient::Vertical,
        }
    }

    pub fn set_orient(&mut self, v: Orient) {
        let v = match v {
            Orient::Horizontal => QtOrientation::Horizontal,
            Orient::Vertical => QtOrientation::Vertical,
        };
        self.widget.pin_mut().setOrientation(v);
    }

    pub fn range(&self) -> (usize, usize) {
        let max = self.widget.as_ref().maximum();
        let min = self.widget.as_ref().minimum();
        (min as _, max as usize + self.page())
    }

    pub fn set_range(&mut self, min: usize, max: usize) {
        self.widget.pin_mut().setMinimum(min as _);
        let page = self.page();
        self.widget
            .pin_mut()
            .setMaximum(max.saturating_sub(page) as _);
    }

    pub fn page(&self) -> usize {
        self.widget.as_ref().pageStep() as _
    }

    pub fn set_page(&mut self, v: usize) {
        self.widget.pin_mut().setPageStep(v as _);
    }

    pub fn pos(&self) -> usize {
        self.widget.as_ref().value() as _
    }

    pub fn set_pos(&mut self, v: usize) {
        self.widget.pin_mut().setValue(v as _);
    }

    fn on_moved(c: *const u8) {
        let c = c as *const Callback<()>;
        if let Some(c) = unsafe { c.as_ref() } {
            c.signal::<GlobalRuntime>(());
        }
    }

    pub async fn wait_change(&self) {
        self.on_moved.wait().await
    }
}

winio_handle::impl_as_widget!(ScrollBar, widget);

impl_static_cast!(ffi::QScrollBar, ffi::QWidget);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
#[non_exhaustive]
pub(crate) enum QtOrientation {
    Horizontal = 0x1,
    Vertical   = 0x2,
}

unsafe impl ExternType for QtOrientation {
    type Id = type_id!("QtOrientation");
    type Kind = cxx::kind::Trivial;
}

#[cxx::bridge]
mod ffi {
    unsafe extern "C++-unwind" {
        include!("winio-ui-qt/src/ui/scroll_bar.hpp");

        type QWidget = crate::ui::QWidget;
        type QScrollBar;
        type QtOrientation = super::QtOrientation;

        unsafe fn new_scroll_bar(parent: *mut QWidget) -> UniquePtr<QScrollBar>;

        unsafe fn scroll_bar_connect_moved(
            w: Pin<&mut QScrollBar>,
            callback: unsafe fn(*const u8),
            data: *const u8,
        );

        fn maximum(self: &QScrollBar) -> i32;
        fn setMaximum(self: Pin<&mut QScrollBar>, v: i32);

        fn minimum(self: &QScrollBar) -> i32;
        fn setMinimum(self: Pin<&mut QScrollBar>, v: i32);

        fn value(self: &QScrollBar) -> i32;
        fn setValue(self: Pin<&mut QScrollBar>, v: i32);

        fn pageStep(self: &QScrollBar) -> i32;
        fn setPageStep(self: Pin<&mut QScrollBar>, v: i32);

        fn orientation(self: &QScrollBar) -> QtOrientation;
        fn setOrientation(self: Pin<&mut QScrollBar>, v: QtOrientation);
    }
}
