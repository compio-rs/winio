use std::{fmt::Debug, mem::ManuallyDrop, pin::Pin};

use cxx::{ExternType, UniquePtr, memory::UniquePtrTarget, type_id};
pub use ffi::is_dark;
use winio_handle::{AsRawWindow, RawWindow};
use winio_primitive::{Point, Rect, Size};

use crate::ui::StaticCastTo;

pub(crate) struct Widget<T: UniquePtrTarget> {
    widget: ManuallyDrop<UniquePtr<T>>,
}

impl<T> Widget<T>
where
    T: UniquePtrTarget + StaticCastTo<ffi::QWidget>,
{
    pub fn new(widget: UniquePtr<T>) -> Self {
        Self {
            widget: ManuallyDrop::new(widget),
        }
    }

    pub(crate) unsafe fn drop_in_place(&mut self) {
        ManuallyDrop::drop(&mut self.widget);
    }

    pub(crate) fn as_ref(&self) -> &T {
        &self.widget
    }

    pub(crate) fn pin_mut(&mut self) -> Pin<&mut T> {
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

    pub fn client_rect(&self) -> Rect {
        let geometry = self.as_ref_qwidget().geometry();
        Rect::new(
            Point::new(geometry.x1 as _, geometry.y1 as _),
            Size::new(
                (geometry.x2 - geometry.x1) as _,
                (geometry.y2 - geometry.y1) as _,
            ),
        )
    }

    pub fn text(&self) -> String {
        self.as_ref_qwidget().windowTitle().into()
    }

    pub fn set_text(&mut self, s: &str) {
        self.pin_mut_qwidget().setWindowTitle(&s.into());
    }
}

impl<T> AsRawWindow for Widget<T>
where
    T: UniquePtrTarget + StaticCastTo<ffi::QWidget>,
{
    fn as_raw_window(&self) -> RawWindow {
        RawWindow::Qt(
            (self.as_ref_qwidget() as *const ffi::QWidget)
                .cast_mut()
                .cast(),
        )
    }
}

impl<T> Debug for Widget<T>
where
    T: UniquePtrTarget,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Widget").finish_non_exhaustive()
    }
}

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

#[cxx::bridge]
mod ffi {
    unsafe extern "C++-unwind" {
        include!("winio-ui-qt/src/ui/widget.hpp");

        fn is_dark() -> bool;

        type QWidget = crate::ui::QWidget;
        type QSize = super::QSize;
        type QRect = super::QRect;
        type QString = crate::ui::QString;

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
    }
}
