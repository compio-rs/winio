use std::{fmt::Debug, pin::Pin};

use cxx::UniquePtr;
use winio_callback::Callback;
use winio_handle::{AsRawContainer, AsRawWindow, RawContainer, RawWindow};
use winio_primitive::{Point, Size};

use crate::{GlobalRuntime, Result, StaticCastTo, ui::impl_static_cast};

pub struct Window {
    on_resize: Box<Callback<Size>>,
    on_move: Box<Callback<Point>>,
    on_close: Box<Callback<()>>,
    on_theme: Box<Callback<()>>,
    widget: UniquePtr<ffi::QMainWindow>,
}

impl Window {
    pub fn new() -> Result<Self> {
        let mut widget = ffi::new_main_window()?;
        let on_resize = Box::new(Callback::new());
        let on_move = Box::new(Callback::new());
        let on_close = Box::new(Callback::new());
        let on_theme = Box::new(Callback::new());
        unsafe {
            ffi::main_window_register_resize_event(
                widget.pin_mut(),
                Self::on_resize,
                on_resize.as_ref() as *const _ as _,
            )?;
            ffi::main_window_register_move_event(
                widget.pin_mut(),
                Self::on_move,
                on_move.as_ref() as *const _ as _,
            )?;
            ffi::main_window_register_close_event(
                widget.pin_mut(),
                Self::on_close,
                on_close.as_ref() as *const _ as _,
            )?;
            ffi::main_window_register_theme_event(
                widget.pin_mut(),
                Self::on_theme,
                on_theme.as_ref() as *const _ as _,
            )?;
        }
        Ok(Self {
            on_resize,
            on_move,
            on_close,
            on_theme,
            widget,
        })
    }

    fn as_ref_qwidget(&self) -> &ffi::QWidget {
        (*self.widget).static_cast()
    }

    fn pin_mut_qwidget(&mut self) -> Pin<&mut ffi::QWidget> {
        self.widget.pin_mut().static_cast_mut()
    }

    pub fn is_visible(&self) -> Result<bool> {
        Ok(self.as_ref_qwidget().isVisible()?)
    }

    pub fn set_visible(&mut self, v: bool) -> Result<()> {
        self.pin_mut_qwidget().setVisible(v)?;
        Ok(())
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

    pub fn client_size(&self) -> Result<Size> {
        let geometry = self.as_ref_qwidget().geometry()?;
        Ok(Size::new(
            (geometry.x2 - geometry.x1) as _,
            (geometry.y2 - geometry.y1) as _,
        ))
    }

    pub fn text(&self) -> Result<String> {
        Ok(self.as_ref_qwidget().windowTitle()?.try_into()?)
    }

    pub fn set_text(&mut self, s: impl AsRef<str>) -> Result<()> {
        self.pin_mut_qwidget()
            .setWindowTitle(&s.as_ref().try_into()?)?;
        Ok(())
    }

    fn on_resize(c: *const u8, width: i32, height: i32) {
        let c = c as *const Callback<Size>;
        if let Some(c) = unsafe { c.as_ref() } {
            c.signal::<GlobalRuntime>(Size::new(width as _, height as _));
        }
    }

    fn on_move(c: *const u8, x: i32, y: i32) {
        let c = c as *const Callback<Point>;
        if let Some(c) = unsafe { c.as_ref() } {
            c.signal::<GlobalRuntime>(Point::new(x as _, y as _));
        }
    }

    fn on_close(c: *const u8) -> bool {
        let c = c as *const Callback<()>;
        if let Some(c) = unsafe { c.as_ref() } {
            if !c.signal::<GlobalRuntime>(()) {
                return true;
            }
        }
        false
    }

    fn on_theme(c: *const u8) {
        let c = c as *const Callback<()>;
        if let Some(c) = unsafe { c.as_ref() } {
            c.signal::<GlobalRuntime>(());
        }
    }

    pub async fn wait_size(&self) {
        self.on_resize.wait().await;
    }

    pub async fn wait_move(&self) {
        self.on_move.wait().await;
    }

    pub async fn wait_close(&self) {
        self.on_close.wait().await
    }

    pub async fn wait_theme_changed(&self) {
        self.on_theme.wait().await
    }
}

impl AsRawWindow for Window {
    fn as_raw_window(&self) -> RawWindow {
        RawWindow::Qt(
            (self.as_ref_qwidget() as *const ffi::QWidget)
                .cast_mut()
                .cast(),
        )
    }
}

winio_handle::impl_as_window!(Window);

impl AsRawContainer for Window {
    fn as_raw_container(&self) -> RawContainer {
        RawContainer::Qt(
            (self.as_ref_qwidget() as *const ffi::QWidget)
                .cast_mut()
                .cast(),
        )
    }
}

winio_handle::impl_as_container!(Window);

impl_static_cast!(ffi::QMainWindow, ffi::QWidget);

impl Debug for Window {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Window").finish_non_exhaustive()
    }
}

#[cxx::bridge]
mod ffi {
    unsafe extern "C++-unwind" {
        include!("winio-ui-qt/src/ui/window.hpp");

        type QWidget = crate::ui::QWidget;
        type QMainWindow;

        fn new_main_window() -> Result<UniquePtr<QMainWindow>>;

        unsafe fn main_window_register_resize_event(
            w: Pin<&mut QMainWindow>,
            callback: unsafe fn(*const u8, i32, i32),
            data: *const u8,
        ) -> Result<()>;
        unsafe fn main_window_register_move_event(
            w: Pin<&mut QMainWindow>,
            callback: unsafe fn(*const u8, i32, i32),
            data: *const u8,
        ) -> Result<()>;
        unsafe fn main_window_register_close_event(
            w: Pin<&mut QMainWindow>,
            callback: unsafe fn(*const u8) -> bool,
            data: *const u8,
        ) -> Result<()>;
        unsafe fn main_window_register_theme_event(
            w: Pin<&mut QMainWindow>,
            callback: unsafe fn(*const u8),
            data: *const u8,
        ) -> Result<()>;
    }
}
