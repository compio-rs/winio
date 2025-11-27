use std::{fmt::Debug, pin::Pin};

use cxx::{ExternType, UniquePtr, memory::UniquePtrTarget, type_id};
use inherit_methods_macro::inherit_methods;
use winio_callback::Callback;
use winio_handle::AsContainer;
use winio_primitive::{Orient, Point, Size, TickPosition};

use crate::{
    GlobalRuntime, Result, StaticCastTo, Widget, impl_static_cast, impl_static_cast_propogate,
};

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
    fn new_impl(mut widget: UniquePtr<T>) -> Result<Self> {
        let on_moved = Box::new(Callback::new());
        unsafe {
            ffi::scroll_bar_connect_moved(
                widget.pin_mut().static_cast_mut(),
                Self::on_moved,
                on_moved.as_ref() as *const _ as _,
            )?;
        }
        let mut widget = Widget::new(widget)?;
        widget.set_visible(true)?;
        Ok(Self { on_moved, widget })
    }

    fn as_abstract_ref(&self) -> &ffi::QAbstractSlider {
        self.widget.as_ref().static_cast()
    }

    fn pin_abstract_mut(&mut self) -> Pin<&mut ffi::QAbstractSlider> {
        self.widget.pin_mut().static_cast_mut()
    }

    pub fn is_visible(&self) -> Result<bool>;

    pub fn set_visible(&mut self, v: bool) -> Result<()>;

    pub fn is_enabled(&self) -> Result<bool>;

    pub fn set_enabled(&mut self, v: bool) -> Result<()>;

    pub fn preferred_size(&self) -> Result<Size>;

    pub fn loc(&self) -> Result<Point>;

    pub fn set_loc(&mut self, p: Point) -> Result<()>;

    pub fn size(&self) -> Result<Size>;

    pub fn set_size(&mut self, s: Size) -> Result<()>;

    pub fn tooltip(&self) -> Result<String>;

    pub fn set_tooltip(&mut self, s: impl AsRef<str>) -> Result<()>;

    pub fn orient(&self) -> Result<Orient> {
        match self.as_abstract_ref().orientation()? {
            QtOrientation::Horizontal => Ok(Orient::Horizontal),
            QtOrientation::Vertical => Ok(Orient::Vertical),
        }
    }

    pub fn set_orient(&mut self, v: Orient) -> Result<()> {
        let v = match v {
            Orient::Horizontal => QtOrientation::Horizontal,
            Orient::Vertical => QtOrientation::Vertical,
        };
        self.pin_abstract_mut().setOrientation(v)?;
        Ok(())
    }

    pub fn minimum(&self) -> Result<usize> {
        Ok(self.as_abstract_ref().minimum()? as _)
    }

    pub fn set_minimum(&mut self, v: usize) -> Result<()> {
        self.pin_abstract_mut().setMinimum(v as _)?;
        Ok(())
    }

    pub fn maximum(&self) -> Result<usize> {
        Ok(self.as_abstract_ref().maximum()? as usize + self.page()?)
    }

    pub fn set_maximum(&mut self, v: usize) -> Result<()> {
        let page = self.page()?;
        self.pin_abstract_mut()
            .setMaximum(v.saturating_sub(page) as _)?;
        Ok(())
    }

    pub fn page(&self) -> Result<usize> {
        Ok(self.as_abstract_ref().pageStep()? as _)
    }

    pub fn set_page(&mut self, v: usize) -> Result<()> {
        self.pin_abstract_mut().setPageStep(v as _)?;
        Ok(())
    }

    pub fn pos(&self) -> Result<usize> {
        Ok(self.as_abstract_ref().value()? as _)
    }

    pub fn set_pos(&mut self, v: usize) -> Result<()> {
        self.pin_abstract_mut().setValue(v as _)?;
        Ok(())
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
    pub fn new(parent: impl AsContainer) -> Result<Self> {
        let widget = unsafe { ffi::new_scroll_bar(parent.as_container().as_qt()) }?;
        Self::new_impl(widget)
    }
}

impl Slider {
    pub fn new(parent: impl AsContainer) -> Result<Self> {
        let widget = unsafe { ffi::new_slider(parent.as_container().as_qt()) }?;
        Self::new_impl(widget)
    }

    pub fn freq(&self) -> Result<usize> {
        Ok(self.widget.as_ref().tickInterval()? as _)
    }

    pub fn set_freq(&mut self, v: usize) -> Result<()> {
        self.widget.pin_mut().setTickInterval(v as _)?;
        Ok(())
    }

    pub fn tick_pos(&self) -> Result<TickPosition> {
        Ok(self.widget.as_ref().tickPosition()?.into())
    }

    pub fn set_tick_pos(&mut self, v: TickPosition) -> Result<()> {
        self.widget.pin_mut().setTickPosition(v.into())?;
        Ok(())
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

        unsafe fn new_scroll_bar(parent: *mut QWidget) -> Result<UniquePtr<QScrollBar>>;
        unsafe fn new_slider(parent: *mut QWidget) -> Result<UniquePtr<QSlider>>;

        unsafe fn scroll_bar_connect_moved(
            w: Pin<&mut QAbstractSlider>,
            callback: unsafe fn(*const u8, Pin<&mut QAbstractSlider>),
            data: *const u8,
        ) -> Result<()>;

        fn maximum(self: &QAbstractSlider) -> Result<i32>;
        fn setMaximum(self: Pin<&mut QAbstractSlider>, v: i32) -> Result<()>;

        fn minimum(self: &QAbstractSlider) -> Result<i32>;
        fn setMinimum(self: Pin<&mut QAbstractSlider>, v: i32) -> Result<()>;

        fn value(self: &QAbstractSlider) -> Result<i32>;
        fn setValue(self: Pin<&mut QAbstractSlider>, v: i32) -> Result<()>;

        fn pageStep(self: &QAbstractSlider) -> Result<i32>;
        fn setPageStep(self: Pin<&mut QAbstractSlider>, v: i32) -> Result<()>;

        fn orientation(self: &QAbstractSlider) -> Result<QtOrientation>;
        fn setOrientation(self: Pin<&mut QAbstractSlider>, v: QtOrientation) -> Result<()>;

        fn tickInterval(self: &QSlider) -> Result<i32>;
        fn setTickInterval(self: Pin<&mut QSlider>, v: i32) -> Result<()>;

        fn tickPosition(self: &QSlider) -> Result<QSliderTickPosition>;
        fn setTickPosition(self: Pin<&mut QSlider>, v: QSliderTickPosition) -> Result<()>;
    }
}
