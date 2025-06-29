use std::ptr::null;

use windows_sys::Win32::{
    Graphics::Gdi::InvalidateRect,
    System::SystemServices::{SS_CENTER, SS_LEFT, SS_RIGHT},
    UI::{
        Controls::WC_STATICW,
        WindowsAndMessaging::{WS_CHILD, WS_EX_TRANSPARENT, WS_VISIBLE},
    },
};
use winio_handle::{AsRawWindow, AsWindow};
use winio_primitive::{HAlign, Point, Size};

use crate::ui::{Widget, font::measure_string};

#[derive(Debug)]
pub struct Label {
    handle: Widget,
}

impl Label {
    pub fn new(parent: impl AsWindow) -> Self {
        let mut handle = Widget::new(
            WC_STATICW,
            WS_CHILD | WS_VISIBLE | SS_LEFT,
            0,
            parent.as_window().as_raw_window(),
        );
        handle.set_size(handle.size_d2l((100, 50)));
        Self { handle }
    }

    pub fn is_visible(&self) -> bool {
        self.handle.is_visible()
    }

    pub fn set_visible(&mut self, v: bool) {
        self.handle.set_visible(v);
    }

    pub fn is_enabled(&self) -> bool {
        self.handle.is_enabled()
    }

    pub fn set_enabled(&mut self, v: bool) {
        self.handle.set_enabled(v);
    }

    pub fn preferred_size(&self) -> Size {
        measure_string(self.handle.as_raw_window(), &self.handle.text_u16())
    }

    pub fn loc(&self) -> Point {
        self.handle.loc()
    }

    pub fn set_loc(&mut self, p: Point) {
        self.handle.set_loc(p)
    }

    pub fn size(&self) -> Size {
        self.handle.size()
    }

    pub fn set_size(&mut self, v: Size) {
        self.handle.set_size(v);
        unsafe { InvalidateRect(self.handle.as_raw_window(), null(), 1) };
    }

    pub fn text(&self) -> String {
        self.handle.text()
    }

    pub fn set_text(&mut self, s: impl AsRef<str>) {
        self.handle.set_text(s)
    }

    pub fn halign(&self) -> HAlign {
        let style = self.handle.style();
        if (style & SS_RIGHT) == SS_RIGHT {
            HAlign::Right
        } else if (style & SS_CENTER) == SS_CENTER {
            HAlign::Center
        } else {
            HAlign::Left
        }
    }

    pub fn set_halign(&mut self, align: HAlign) {
        let mut style = self.handle.style();
        style &= !(SS_RIGHT);
        match align {
            HAlign::Center => style |= SS_CENTER,
            HAlign::Right => style |= SS_RIGHT,
            _ => style |= SS_LEFT,
        }
        self.handle.set_style(style)
    }

    pub fn is_transparent(&self) -> bool {
        (self.handle.ex_style() & WS_EX_TRANSPARENT) != 0
    }

    pub fn set_transparent(&mut self, v: bool) {
        let style = if v { WS_EX_TRANSPARENT } else { 0 };
        self.handle.set_ex_style(style);
    }
}
