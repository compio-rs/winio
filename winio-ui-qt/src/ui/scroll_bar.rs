use std::{fmt::Debug, pin::Pin};

use cxx::{ExternType, UniquePtr, memory::UniquePtrTarget, type_id};
use inherit_methods_macro::inherit_methods;
use winio_callback::Callback;
use winio_handle::AsContainer;
use winio_primitive::{Orient, Point, Size, TickPosition};

use crate::{GlobalRuntime, StaticCastTo, Widget, impl_static_cast, impl_static_cast_propogate};

pub struct ScrollBarImpl<T>
where
    T: UniquePtrTarget + StaticCastTo<ffi::QWidget>,
{
    on_moved: Box<Callback>,
    widget: Widget<T>,
}

#[allow(private_bounds)]
#[inherit_methods(from = "self.widget")]
impl<T> ScrollBarImpl<T>
where
    T: StaticCastTo<ffi::QAbstractSlider> + StaticCastTo<ffi::QWidget> + UniquePtrTarget,
{
    fn new_impl(mut widget: UniquePtr<T>) -> Self {
        let on_moved = Box::new(Callback::new());
        unsafe {
            ffi::scroll_bar_connect_moved(
                widget.pin_mut().static_cast_mut(),
                Self::on_moved,
                on_moved.as_ref() as *const _ as _,
            );
        }
        let mut widget = Widget::new(widget);
        widget.set_visible(true);
        Self { on_moved, widget }
    }

    fn as_abstract_ref(&self) -> &ffi::QAbstractSlider {
        self.widget.as_ref().static_cast()
    }

    fn pin_abstract_mut(&mut self) -> Pin<&mut ffi::QAbstractSlider> {
        self.widget.pin_mut().static_cast_mut()
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

    pub fn tooltip(&self) -> String;

    pub fn set_tooltip(&mut self, s: impl AsRef<str>);

    pub fn orient(&self) -> Orient {
        match self.as_abstract_ref().orientation() {
            QtOrientation::Horizontal => Orient::Horizontal,
            QtOrientation::Vertical => Orient::Vertical,
        }
    }

    pub fn set_orient(&mut self, v: Orient) {
        let v = match v {
            Orient::Horizontal => QtOrientation::Horizontal,
            Orient::Vertical => QtOrientation::Vertical,
        };
        self.pin_abstract_mut().setOrientation(v);
    }

    pub fn minimum(&self) -> usize {
        self.as_abstract_ref().minimum() as _
    }

    pub fn set_minimum(&mut self, v: usize) {
        self.pin_abstract_mut().setMinimum(v as _);
    }

    pub fn maximum(&self) -> usize {
        self.as_abstract_ref().maximum() as usize + self.page()
    }

    pub fn set_maximum(&mut self, v: usize) {
        let page = self.page();
        self.pin_abstract_mut()
            .setMaximum(v.saturating_sub(page) as _);
    }

    pub fn page(&self) -> usize {
        self.as_abstract_ref().pageStep() as _
    }

    pub fn set_page(&mut self, v: usize) {
        self.pin_abstract_mut().setPageStep(v as _);
    }

    pub fn pos(&self) -> usize {
        self.as_abstract_ref().value() as _
    }

    pub fn set_pos(&mut self, v: usize) {
        self.pin_abstract_mut().setValue(v as _);
    }

    fn on_moved(c: *const u8, _slider: Pin<&mut ffi::QAbstractSlider>) {
        let c = c as *const Callback<()>;
        if let Some(c) = unsafe { c.as_ref() } {
            c.signal::<GlobalRuntime>(());
        }
    }

    pub async fn wait_change(&self) {
        self.on_moved.wait().await
    }
}

impl<T: UniquePtrTarget + StaticCastTo<ffi::QWidget>> Debug for ScrollBarImpl<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ScrollBarImpl")
            .field("on_moved", &self.on_moved)
            .field("widget", &self.widget)
            .finish()
    }
}

pub type ScrollBar = ScrollBarImpl<ffi::QScrollBar>;
pub type Slider = ScrollBarImpl<ffi::QSlider>;

winio_handle::impl_as_widget!(ScrollBar, widget);
winio_handle::impl_as_widget!(Slider, widget);

impl ScrollBar {
    pub fn new(parent: impl AsContainer) -> Self {
        let widget = unsafe { ffi::new_scroll_bar(parent.as_container().as_qt()) };
        Self::new_impl(widget)
    }
}

impl Slider {
    pub fn new(parent: impl AsContainer) -> Self {
        let widget = unsafe { ffi::new_slider(parent.as_container().as_qt()) };
        Self::new_impl(widget)
    }

    pub fn freq(&self) -> usize {
        self.widget.as_ref().tickInterval() as _
    }

    pub fn set_freq(&mut self, v: usize) {
        self.widget.pin_mut().setTickInterval(v as _);
    }

    pub fn tick_pos(&self) -> TickPosition {
        self.widget.as_ref().tickPosition().into()
    }

    pub fn set_tick_pos(&mut self, v: TickPosition) {
        self.widget.pin_mut().setTickPosition(v.into());
    }
}

impl_static_cast!(ffi::QAbstractSlider, ffi::QWidget);

impl_static_cast!(ffi::QScrollBar, ffi::QAbstractSlider);

impl_static_cast_propogate!(ffi::QScrollBar, ffi::QAbstractSlider, ffi::QWidget);

impl_static_cast!(ffi::QSlider, ffi::QAbstractSlider);

impl_static_cast_propogate!(ffi::QSlider, ffi::QAbstractSlider, ffi::QWidget);

#[doc(hidden)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
#[non_exhaustive]
pub enum QtOrientation {
    Horizontal = 0x1,
    Vertical   = 0x2,
}

unsafe impl ExternType for QtOrientation {
    type Id = type_id!("QtOrientation");
    type Kind = cxx::kind::Trivial;
}

#[doc(hidden)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
#[non_exhaustive]
pub enum QSliderTickPosition {
    NoTicks        = 0,
    TicksAbove     = 1,
    TicksBelow     = 2,
    TicksBothSides = 3,
}

unsafe impl ExternType for QSliderTickPosition {
    type Id = type_id!("QSliderTickPosition");
    type Kind = cxx::kind::Trivial;
}

impl From<QSliderTickPosition> for TickPosition {
    fn from(v: QSliderTickPosition) -> Self {
        match v {
            QSliderTickPosition::NoTicks => TickPosition::None,
            QSliderTickPosition::TicksAbove => TickPosition::TopLeft,
            QSliderTickPosition::TicksBelow => TickPosition::BottomRight,
            QSliderTickPosition::TicksBothSides => TickPosition::Both,
        }
    }
}

impl From<TickPosition> for QSliderTickPosition {
    fn from(v: TickPosition) -> Self {
        match v {
            TickPosition::None => QSliderTickPosition::NoTicks,
            TickPosition::TopLeft => QSliderTickPosition::TicksAbove,
            TickPosition::BottomRight => QSliderTickPosition::TicksBelow,
            TickPosition::Both => QSliderTickPosition::TicksBothSides,
        }
    }
}

#[cxx::bridge]
mod ffi {
    unsafe extern "C++-unwind" {
        include!("winio-ui-qt/src/ui/scroll_bar.hpp");

        type QWidget = crate::ui::QWidget;
        type QAbstractSlider;
        type QScrollBar;
        type QSlider;
        type QtOrientation = super::QtOrientation;
        type QSliderTickPosition = super::QSliderTickPosition;

        unsafe fn new_scroll_bar(parent: *mut QWidget) -> UniquePtr<QScrollBar>;
        unsafe fn new_slider(parent: *mut QWidget) -> UniquePtr<QSlider>;

        unsafe fn scroll_bar_connect_moved(
            w: Pin<&mut QAbstractSlider>,
            callback: unsafe fn(*const u8, Pin<&mut QAbstractSlider>),
            data: *const u8,
        );

        fn maximum(self: &QAbstractSlider) -> i32;
        fn setMaximum(self: Pin<&mut QAbstractSlider>, v: i32);

        fn minimum(self: &QAbstractSlider) -> i32;
        fn setMinimum(self: Pin<&mut QAbstractSlider>, v: i32);

        fn value(self: &QAbstractSlider) -> i32;
        fn setValue(self: Pin<&mut QAbstractSlider>, v: i32);

        fn pageStep(self: &QAbstractSlider) -> i32;
        fn setPageStep(self: Pin<&mut QAbstractSlider>, v: i32);

        fn orientation(self: &QAbstractSlider) -> QtOrientation;
        fn setOrientation(self: Pin<&mut QAbstractSlider>, v: QtOrientation);

        fn tickInterval(self: &QSlider) -> i32;
        fn setTickInterval(self: Pin<&mut QSlider>, v: i32);

        fn tickPosition(self: &QSlider) -> QSliderTickPosition;
        fn setTickPosition(self: Pin<&mut QSlider>, v: QSliderTickPosition);
    }
}
