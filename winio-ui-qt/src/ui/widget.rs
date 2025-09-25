use std::{
    fmt::Debug,
    mem::{ManuallyDrop, MaybeUninit},
    pin::Pin,
};

use cxx::{ExternType, UniquePtr, memory::UniquePtrTarget, type_id};
pub use ffi::{QWidget, is_dark};
use inherit_methods_macro::inherit_methods;
use winio_handle::{AsContainer, AsRawContainer, AsRawWidget, RawContainer, RawWidget};
use winio_primitive::{Point, Size};

use crate::ui::StaticCastTo;

pub(crate) struct Widget<T: UniquePtrTarget + StaticCastTo<ffi::QWidget>> {
    widget: ManuallyDrop<UniquePtr<T>>,
    weak_ref: QWidgetPointer,
}

impl<T> Widget<T>
where
    T: UniquePtrTarget + StaticCastTo<ffi::QWidget>,
{
    pub fn new(mut widget: UniquePtr<T>) -> Self {
        let weak_ref =
            unsafe { ffi::widget_weak(widget.pin_mut().static_cast_mut().get_unchecked_mut()) };
        Self {
            widget: ManuallyDrop::new(widget),
            weak_ref,
        }
    }

    #[inline]
    fn check_ref(&self) {
        #[cold]
        fn panic_null() {
            unreachable!("the widget has been deleted by its parent")
        }
        if self.weak_ref.isNull() {
            panic_null();
        }
    }

    pub(crate) fn as_ref(&self) -> &T {
        self.check_ref();
        &self.widget
    }

    pub(crate) fn pin_mut(&mut self) -> Pin<&mut T> {
        self.check_ref();
        self.widget.pin_mut()
    }

    fn as_ref_qwidget(&self) -> &ffi::QWidget {
        self.as_ref().static_cast()
    }

    fn pin_mut_qwidget(&mut self) -> Pin<&mut ffi::QWidget> {
        self.pin_mut().static_cast_mut()
    }

    pub fn is_visible(&self) -> bool {
        self.as_ref_qwidget().isVisible()
    }

    pub fn set_visible(&mut self, v: bool) {
        self.pin_mut_qwidget().setVisible(v);
    }

    pub fn is_enabled(&self) -> bool {
        self.as_ref_qwidget().isEnabled()
    }

    pub fn set_enabled(&mut self, v: bool) {
        self.pin_mut_qwidget().setEnabled(v);
    }

    pub fn preferred_size(&self) -> Size {
        let s = self.as_ref_qwidget().sizeHint();
        Size::new(s.width as _, s.height as _)
    }

    pub fn min_size(&self) -> Size {
        let s = self.as_ref_qwidget().minimumSize();
        Size::new(s.width as _, s.height as _)
    }

    pub fn loc(&self) -> Point {
        Point::new(
            self.as_ref_qwidget().x() as _,
            self.as_ref_qwidget().y() as _,
        )
    }

    pub fn set_loc(&mut self, p: Point) {
        self.pin_mut_qwidget().move_(p.x as _, p.y as _);
    }

    pub fn size(&self) -> Size {
        Size::new(
            self.as_ref_qwidget().width() as _,
            self.as_ref_qwidget().height() as _,
        )
    }

    pub fn set_size(&mut self, s: Size) {
        self.pin_mut_qwidget().resize(s.width as _, s.height as _);
    }

    pub fn tooltip(&self) -> String {
        self.as_ref_qwidget().toolTip().into()
    }

    pub fn set_tooltip(&mut self, s: impl AsRef<str>) {
        self.pin_mut_qwidget().setToolTip(&s.as_ref().into());
    }
}

impl<T> Drop for Widget<T>
where
    T: UniquePtrTarget + StaticCastTo<ffi::QWidget>,
{
    fn drop(&mut self) {
        if !self.weak_ref.isNull() {
            self.pin_mut_qwidget().deleteLater();
        }
    }
}

impl<T> AsRawWidget for Widget<T>
where
    T: UniquePtrTarget + StaticCastTo<ffi::QWidget>,
{
    fn as_raw_widget(&self) -> RawWidget {
        RawWidget::Qt(
            (self.as_ref_qwidget() as *const ffi::QWidget)
                .cast_mut()
                .cast(),
        )
    }
}

impl<T> AsRawContainer for Widget<T>
where
    T: UniquePtrTarget + StaticCastTo<ffi::QWidget>,
{
    fn as_raw_container(&self) -> RawContainer {
        RawContainer::Qt(
            (self.as_ref_qwidget() as *const ffi::QWidget)
                .cast_mut()
                .cast(),
        )
    }
}

impl<T> Debug for Widget<T>
where
    T: UniquePtrTarget + StaticCastTo<ffi::QWidget>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Widget").finish_non_exhaustive()
    }
}

#[derive(Debug)]
pub struct View {
    pub(crate) widget: Widget<ffi::QWidget>,
}

#[inherit_methods(from = "self.widget")]
impl View {
    pub fn new(parent: impl AsContainer) -> Self {
        Self::new_impl(parent.as_container().as_qt())
    }

    pub(crate) fn new_standalone() -> Self {
        Self::new_impl(std::ptr::null_mut())
    }

    fn new_impl(parent: *mut ffi::QWidget) -> Self {
        let widget = unsafe { ffi::new_widget(parent) };
        let mut widget = Widget::new(widget);
        widget.set_visible(true);
        Self { widget }
    }

    pub fn is_visible(&self) -> bool;

    pub fn set_visible(&mut self, v: bool);

    pub fn loc(&self) -> Point;

    pub fn set_loc(&mut self, p: Point);

    pub fn size(&self) -> Size;

    pub fn set_size(&mut self, s: Size);
}

winio_handle::impl_as_widget!(View, widget);
winio_handle::impl_as_container!(View, widget);

#[repr(C)]
pub struct QSize {
    pub width: i32,
    pub height: i32,
}

unsafe impl ExternType for QSize {
    type Id = type_id!("QSize");
    type Kind = cxx::kind::Trivial;
}

#[repr(C)]
pub struct QRect {
    pub x1: i32,
    pub y1: i32,
    pub x2: i32,
    pub y2: i32,
}

unsafe impl ExternType for QRect {
    type Id = type_id!("QRect");
    type Kind = cxx::kind::Trivial;
}

#[repr(C)]
pub struct QWidgetPointer {
    _data: MaybeUninit<[usize; 2]>,
}

unsafe impl ExternType for QWidgetPointer {
    type Id = type_id!("QWidgetPointer");
    type Kind = cxx::kind::Trivial;
}

#[cxx::bridge]
mod ffi {
    unsafe extern "C++-unwind" {
        include!("winio-ui-qt/src/ui/widget.hpp");

        fn is_dark() -> bool;

        type QWidget;
        type QSize = super::QSize;
        type QRect = super::QRect;
        type QString = crate::ui::QString;
        type QWidgetPointer = super::QWidgetPointer;

        unsafe fn new_widget(parent: *mut QWidget) -> UniquePtr<QWidget>;
        unsafe fn widget_weak(w: *mut QWidget) -> QWidgetPointer;

        fn isNull(self: &QWidgetPointer) -> bool;

        fn parentWidget(self: &QWidget) -> *mut QWidget;
        fn x(self: &QWidget) -> i32;
        fn y(self: &QWidget) -> i32;
        #[cxx_name = "move"]
        fn move_(self: Pin<&mut QWidget>, x: i32, y: i32);
        fn width(self: &QWidget) -> i32;
        fn height(self: &QWidget) -> i32;
        fn resize(self: Pin<&mut QWidget>, w: i32, h: i32);
        fn geometry(self: &QWidget) -> &QRect;
        fn sizeHint(self: &QWidget) -> QSize;
        fn minimumSize(self: &QWidget) -> QSize;
        fn update(self: Pin<&mut QWidget>);
        fn isVisible(self: &QWidget) -> bool;
        fn setVisible(self: Pin<&mut QWidget>, v: bool);
        fn isEnabled(self: &QWidget) -> bool;
        fn setEnabled(self: Pin<&mut QWidget>, v: bool);
        fn windowTitle(self: &QWidget) -> QString;
        fn setWindowTitle(self: Pin<&mut QWidget>, s: &QString);
        fn toolTip(self: &QWidget) -> QString;
        fn setToolTip(self: Pin<&mut QWidget>, s: &QString);
        fn childrenRect(self: &QWidget) -> QRect;
        fn deleteLater(self: Pin<&mut QWidget>);
        unsafe fn setParent(self: Pin<&mut QWidget>, parent: *mut QWidget);
    }
}
