use inherit_methods_macro::inherit_methods;
use windows_sys::Win32::UI::{
    Controls::{
        PBM_GETPOS, PBM_GETRANGE, PBM_SETMARQUEE, PBM_SETPOS, PBM_SETRANGE32, PBS_MARQUEE,
        PBS_SMOOTHREVERSE, PROGRESS_CLASSW,
    },
    HiDpi::GetSystemMetricsForDpi,
    WindowsAndMessaging::{SM_CYVSCROLL, USER_DEFAULT_SCREEN_DPI, WS_CHILD, WS_VISIBLE},
};
use winio_handle::AsContainer;
use winio_primitive::{Point, Size};

use crate::ui::Widget;

#[derive(Debug)]
pub struct Progress {
    handle: Widget,
}

#[inherit_methods(from = "self.handle")]
impl Progress {
    pub fn new(parent: impl AsContainer) -> Self {
        let mut handle = Widget::new(
            PROGRESS_CLASSW,
            WS_CHILD | WS_VISIBLE | PBS_SMOOTHREVERSE,
            0,
            parent.as_container().as_win32(),
        );
        handle.set_size(handle.size_d2l((100, 15)));
        Self { handle }
    }

    pub fn is_visible(&self) -> bool;

    pub fn set_visible(&mut self, v: bool);

    pub fn is_enabled(&self) -> bool;

    pub fn set_enabled(&mut self, v: bool);

    pub fn preferred_size(&self) -> Size {
        let height = unsafe { GetSystemMetricsForDpi(SM_CYVSCROLL, USER_DEFAULT_SCREEN_DPI) };
        Size::new(0.0, height as _)
    }

    pub fn loc(&self) -> Point;

    pub fn set_loc(&mut self, p: Point);

    pub fn size(&self) -> Size;

    pub fn set_size(&mut self, v: Size);

    pub fn tooltip(&self) -> String;

    pub fn set_tooltip(&mut self, s: impl AsRef<str>);

    pub fn minimum(&self) -> usize {
        self.handle.send_message(PBM_GETRANGE, 1, 0) as usize
    }

    pub fn maximum(&self) -> usize {
        self.handle.send_message(PBM_GETRANGE, 0, 0) as usize
    }

    pub fn set_minimum(&mut self, v: usize) {
        self.handle
            .send_message(PBM_SETRANGE32, v as _, self.maximum() as _);
    }

    pub fn set_maximum(&mut self, v: usize) {
        self.handle
            .send_message(PBM_SETRANGE32, self.minimum() as _, v as _);
    }

    pub fn pos(&self) -> usize {
        self.handle.send_message(PBM_GETPOS, 0, 0) as _
    }

    pub fn set_pos(&mut self, pos: usize) {
        self.handle.send_message(PBM_SETPOS, pos as _, 0);
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
        self.handle
            .send_message(PBM_SETMARQUEE, if v { 1 } else { 0 }, 0);
    }
}

winio_handle::impl_as_widget!(Progress, handle);
