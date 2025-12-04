use std::{
    fmt::Debug,
    mem::{ManuallyDrop, MaybeUninit},
    pin::Pin,
};

use compio_log::error;
use cxx::{ExternType, UniquePtr, memory::UniquePtrTarget, type_id};
pub use ffi::{QWidget, is_dark};
use inherit_methods_macro::inherit_methods;
use winio_handle::{AsContainer, AsWidget, BorrowedContainer, BorrowedWidget};
use winio_primitive::{Point, Size};

use crate::{Result, ui::StaticCastTo};

pub(crate) struct Widget<T: UniquePtrTarget + StaticCastTo<ffi::QWidget>> {
    widget: ManuallyDrop<UniquePtr<T>>,
    weak_ref: QWidgetPointer,
}

impl<T> Widget<T>
where
    T: UniquePtrTarget + StaticCastTo<ffi::QWidget>,
{
    pub fn new(mut widget: UniquePtr<T>) -> Result<Self> {
        let weak_ref =
            unsafe { ffi::widget_weak(widget.pin_mut().static_cast_mut().get_unchecked_mut())? };
        Ok(Self {
            widget: ManuallyDrop::new(widget),
            weak_ref,
        })
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

    pub fn is_visible(&self) -> Result<bool> {
        Ok(self.as_ref_qwidget().isVisible()?)
    }

    pub fn set_visible(&mut self, v: bool) -> Result<()> {
        self.pin_mut_qwidget().setVisible(v)?;
        Ok(())
    }

    pub fn is_enabled(&self) -> Result<bool> {
        Ok(self.as_ref_qwidget().isEnabled()?)
    }

    pub fn set_enabled(&mut self, v: bool) -> Result<()> {
        self.pin_mut_qwidget().setEnabled(v)?;
        Ok(())
    }

    pub fn preferred_size(&self) -> Result<Size> {
        let s = self.as_ref_qwidget().sizeHint()?;
        Ok(Size::new(s.width as _, s.height as _))
    }

    pub fn min_size(&self) -> Result<Size> {
        let s = self.as_ref_qwidget().minimumSize()?;
        Ok(Size::new(s.width as _, s.height as _))
    }

    pub fn loc(&self) -> Result<Point> {
        let rect = self.as_ref_qwidget().rect()?;
        Ok(Point::new(rect.x1 as _, rect.y1 as _))
    }

    pub fn set_loc(&mut self, p: Point) -> Result<()> {
        self.pin_mut_qwidget().move_(p.x as _, p.y as _)?;
        Ok(())
    }

    pub fn size(&self) -> Result<Size> {
        let rect = self.as_ref_qwidget().rect()?;
        Ok(Size::new(
            (rect.x2 - rect.x1) as _,
            (rect.y2 - rect.y1) as _,
        ))
    }

    pub fn set_size(&mut self, s: Size) -> Result<()> {
        self.pin_mut_qwidget().resize(s.width as _, s.height as _)?;
        Ok(())
    }

    pub fn tooltip(&self) -> Result<String> {
        Ok(self.as_ref_qwidget().toolTip()?.try_into()?)
    }

    pub fn set_tooltip(&mut self, s: impl AsRef<str>) -> Result<()> {
        self.pin_mut_qwidget().setToolTip(&s.as_ref().try_into()?)?;
        Ok(())
    }
}

impl<T> Drop for Widget<T>
where
    T: UniquePtrTarget + StaticCastTo<ffi::QWidget>,
{
    fn drop(&mut self) {
        if !self.weak_ref.isNull()
            && let Err(_e) = self.pin_mut_qwidget().deleteLater()
        {
            error!("Failed to delete widget later: {_e:?}");
        }
    }
}

impl<T> AsWidget for Widget<T>
where
    T: UniquePtrTarget + StaticCastTo<ffi::QWidget>,
{
    fn as_widget(&self) -> BorrowedWidget<'_> {
        unsafe { BorrowedWidget::qt((self.as_ref_qwidget() as *const ffi::QWidget).cast_mut()) }
    }
}

impl<T> AsContainer for Widget<T>
where
    T: UniquePtrTarget + StaticCastTo<ffi::QWidget>,
{
    fn as_container(&self) -> BorrowedContainer<'_> {
        unsafe { BorrowedContainer::qt((self.as_ref_qwidget() as *const ffi::QWidget).cast_mut()) }
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
    pub fn new(parent: impl AsContainer) -> Result<Self> {
        Self::new_impl(parent.as_container().as_qt())
    }

    pub(crate) fn new_standalone() -> Result<Self> {
        Self::new_impl(std::ptr::null_mut())
    }

    fn new_impl(parent: *mut ffi::QWidget) -> Result<Self> {
        let widget = unsafe { ffi::new_widget(parent) }?;
        let mut widget = Widget::new(widget)?;
        widget.set_visible(true)?;
        Ok(Self { widget })
    }

    pub fn is_visible(&self) -> Result<bool>;

    pub fn set_visible(&mut self, v: bool) -> Result<()>;

    pub fn loc(&self) -> Result<Point>;

    pub fn set_loc(&mut self, p: Point) -> Result<()>;

    pub fn size(&self) -> Result<Size>;

    pub fn set_size(&mut self, s: Size) -> Result<()>;
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

        fn is_dark() -> Result<bool>;

        type QWidget;
        type QSize = super::QSize;
        type QRect = super::QRect;
        type QString = crate::ui::QString;
        type QWidgetPointer = super::QWidgetPointer;

        unsafe fn new_widget(parent: *mut QWidget) -> Result<UniquePtr<QWidget>>;
        unsafe fn widget_weak(w: *mut QWidget) -> Result<QWidgetPointer>;

        fn isNull(self: &QWidgetPointer) -> bool;

        fn parentWidget(self: &QWidget) -> Result<*mut QWidget>;
        fn rect(self: &QWidget) -> Result<QRect>;
        #[cxx_name = "move"]
        fn move_(self: Pin<&mut QWidget>, x: i32, y: i32) -> Result<()>;
        fn resize(self: Pin<&mut QWidget>, w: i32, h: i32) -> Result<()>;
        fn geometry(self: &QWidget) -> Result<&QRect>;
        fn sizeHint(self: &QWidget) -> Result<QSize>;
        fn minimumSize(self: &QWidget) -> Result<QSize>;
        fn update(self: Pin<&mut QWidget>) -> Result<()>;
        fn isVisible(self: &QWidget) -> Result<bool>;
        fn setVisible(self: Pin<&mut QWidget>, v: bool) -> Result<()>;
        fn isEnabled(self: &QWidget) -> Result<bool>;
        fn setEnabled(self: Pin<&mut QWidget>, v: bool) -> Result<()>;
        fn windowTitle(self: &QWidget) -> Result<QString>;
        fn setWindowTitle(self: Pin<&mut QWidget>, s: &QString) -> Result<()>;
        fn toolTip(self: &QWidget) -> Result<QString>;
        fn setToolTip(self: Pin<&mut QWidget>, s: &QString) -> Result<()>;
        fn childrenRect(self: &QWidget) -> Result<QRect>;
        fn deleteLater(self: Pin<&mut QWidget>) -> Result<()>;
        unsafe fn setParent(self: Pin<&mut QWidget>, parent: *mut QWidget) -> Result<()>;
    }
}
