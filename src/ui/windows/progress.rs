use windows_sys::Win32::UI::{
    Controls::{
        PBM_GETPOS, PBM_GETRANGE, PBM_SETMARQUEE, PBM_SETPOS, PBM_SETRANGE32, PBS_MARQUEE,
        PBS_SMOOTHREVERSE, PROGRESS_CLASSW,
    },
    HiDpi::GetSystemMetricsForDpi,
    WindowsAndMessaging::{
        SM_CYVSCROLL, SendMessageW, USER_DEFAULT_SCREEN_DPI, WS_CHILD, WS_TABSTOP, WS_VISIBLE,
    },
};

use crate::{AsRawWindow, AsWindow, Point, Size, ui::Widget};

#[derive(Debug)]
pub struct Progress {
    handle: Widget,
}

impl Progress {
    pub fn new(parent: impl AsWindow) -> Self {
        let mut handle = Widget::new(
            PROGRESS_CLASSW,
            WS_CHILD | WS_VISIBLE | WS_TABSTOP | PBS_SMOOTHREVERSE,
            0,
            parent.as_window().as_raw_window(),
        );
        handle.set_size(handle.size_d2l((100, 15)));
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
        let height = unsafe { GetSystemMetricsForDpi(SM_CYVSCROLL, USER_DEFAULT_SCREEN_DPI) };
        Size::new(0.0, height as _)
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

    pub fn range(&self) -> (usize, usize) {
        unsafe {
            let min = SendMessageW(self.handle.as_raw_window(), PBM_GETRANGE, 1, 0) as usize;
            let max = SendMessageW(self.handle.as_raw_window(), PBM_GETRANGE, 0, 0) as usize;
            (min, max)
        }
    }

    pub fn set_range(&mut self, min: usize, max: usize) {
        unsafe {
            SendMessageW(
                self.handle.as_raw_window(),
                PBM_SETRANGE32,
                min as _,
                max as _,
            )
        };
    }

    pub fn pos(&self) -> usize {
        unsafe { SendMessageW(self.handle.as_raw_window(), PBM_GETPOS, 0, 0) as _ }
    }

    pub fn set_pos(&mut self, pos: usize) {
        unsafe { SendMessageW(self.handle.as_raw_window(), PBM_SETPOS, pos as _, 0) };
    }

    pub fn is_indeterminate(&self) -> bool {
        (self.handle.style() & PBS_MARQUEE) != 0
    }

    pub fn set_indeterminate(&mut self, v: bool) {
        let mut style = self.handle.style();
        if v {
            style |= PBS_MARQUEE;
        } else {
            style &= !PBS_MARQUEE;
        }
        self.handle.set_style(style);
        unsafe {
            SendMessageW(
                self.handle.as_raw_window(),
                PBM_SETMARQUEE,
                if v { 1 } else { 0 },
                0,
            )
        };
    }
}
