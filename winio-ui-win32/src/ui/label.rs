use std::{io, ptr::null};

use compio::driver::syscall;
use inherit_methods_macro::inherit_methods;
use windows_sys::Win32::{
    Graphics::Gdi::InvalidateRect,
    System::SystemServices::{SS_CENTER, SS_LEFT, SS_NOTIFY, SS_RIGHT},
    UI::{
        Controls::WC_STATICW,
        WindowsAndMessaging::{WS_CHILD, WS_EX_TRANSPARENT, WS_VISIBLE},
    },
};
use winio_handle::{AsContainer, AsRawWindow};
use winio_primitive::{HAlign, Point, Size};

use crate::ui::Widget;

#[derive(Debug)]
pub struct Label {
    handle: Widget,
}

#[inherit_methods(from = "self.handle")]
impl Label {
    pub fn new(parent: impl AsContainer) -> io::Result<Self> {
        let mut handle = Widget::new(
            WC_STATICW,
            // Without SS_NOTIFY ToolTip won't work
            WS_CHILD | WS_VISIBLE | SS_LEFT | SS_NOTIFY,
            0,
            parent.as_container().as_win32(),
        )?;
        Ok(Self { handle })
    }

    pub fn is_visible(&self) -> io::Result<bool>;

    pub fn set_visible(&mut self, v: bool) -> io::Result<()>;

    pub fn is_enabled(&self) -> io::Result<bool>;

    pub fn set_enabled(&mut self, v: bool) -> io::Result<()>;

    pub fn preferred_size(&self) -> io::Result<Size> {
        self.handle.measure_text()
    }

    pub fn loc(&self) -> io::Result<Point>;

    pub fn set_loc(&mut self, p: Point) -> io::Result<()>;

    pub fn size(&self) -> io::Result<Size>;

    pub fn set_size(&mut self, v: Size) -> io::Result<()> {
        self.handle.set_size(v)?;
        syscall!(
            BOOL,
            InvalidateRect(self.handle.as_raw_window().as_win32(), null(), 1)
        )?;
        Ok(())
    }

    pub fn tooltip(&self) -> io::Result<String>;

    pub fn set_tooltip(&mut self, s: impl AsRef<str>) -> io::Result<()>;

    pub fn text(&self) -> io::Result<String>;

    pub fn set_text(&mut self, s: impl AsRef<str>) -> io::Result<()>;

    pub fn halign(&self) -> io::Result<HAlign> {
        let style = self.handle.style()?;
        let style = if (style & SS_RIGHT) == SS_RIGHT {
            HAlign::Right
        } else if (style & SS_CENTER) == SS_CENTER {
            HAlign::Center
        } else {
            HAlign::Left
        };
        Ok(style)
    }

    pub fn set_halign(&mut self, align: HAlign) -> io::Result<()> {
        let mut style = self.handle.style()?;
        style &= !(SS_RIGHT);
        match align {
            HAlign::Center => style |= SS_CENTER,
            HAlign::Right => style |= SS_RIGHT,
            _ => style |= SS_LEFT,
        }
        self.handle.set_style(style)
    }

    pub fn is_transparent(&self) -> io::Result<bool> {
        Ok((self.handle.ex_style()? & WS_EX_TRANSPARENT) != 0)
    }

    pub fn set_transparent(&mut self, v: bool) -> io::Result<()> {
        let style = if v { WS_EX_TRANSPARENT } else { 0 };
        self.handle.set_ex_style(style)
    }
}

winio_handle::impl_as_widget!(Label, handle);
