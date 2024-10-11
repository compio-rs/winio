use windows_sys::Win32::{
    System::SystemServices::{SS_CENTER, SS_LEFT, SS_RIGHT},
    UI::{
        Controls::WC_STATICW,
        WindowsAndMessaging::{WS_CHILD, WS_VISIBLE},
    },
};

use crate::{AsRawWindow, AsWindow, HAlign, Point, Size, ui::Widget};

#[derive(Debug)]
pub struct Label {
    handle: Widget,
}

impl Label {
    pub fn new(parent: impl AsWindow) -> Self {
        let handle = Widget::new(
            WC_STATICW,
            WS_CHILD | WS_VISIBLE | SS_LEFT,
            0,
            parent.as_window().as_raw_window(),
        );
        handle.set_size(handle.size_d2l((100, 50)));
        Self { handle }
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
        self.handle.set_size(v)
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
            HAlign::Left => style |= SS_LEFT,
            HAlign::Center => style |= SS_CENTER,
            HAlign::Right => style |= SS_RIGHT,
        }
        self.handle.set_style(style)
    }
}
