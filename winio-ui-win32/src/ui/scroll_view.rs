use std::{
    mem::{MaybeUninit, size_of},
    ptr::null_mut,
};

use compio::driver::syscall;
use futures_util::FutureExt;
use inherit_methods_macro::inherit_methods;
use windows_sys::{
    Win32::{
        Foundation::{HWND, LPARAM, RECT, WPARAM},
        Graphics::Gdi::MapWindowPoints,
        UI::{
            Controls::{SetScrollInfo, ShowScrollBar},
            WindowsAndMessaging::{
                EnumChildWindows, GetParent, GetScrollInfo, GetWindowRect, HWND_DESKTOP, SB_BOTTOM,
                SB_HORZ, SB_LINEDOWN, SB_LINEUP, SB_PAGEDOWN, SB_PAGEUP, SB_THUMBTRACK, SB_TOP,
                SB_VERT, SCROLLINFO, SIF_ALL, SIF_PAGE, SIF_POS, SIF_RANGE, SWP_NOSIZE,
                SWP_NOZORDER, SetWindowPos, WM_HSCROLL, WM_VSCROLL, WS_CHILDWINDOW,
                WS_EX_CONTROLPARENT, WS_HSCROLL, WS_VISIBLE, WS_VSCROLL,
            },
        },
    },
    core::BOOL,
};
use winio_handle::{AsRawWindow, AsWindow};
use winio_primitive::{Point, Size};

use crate::{View, ui::Widget, window_class_name};

#[derive(Debug)]
pub struct ScrollView {
    handle: Widget,
    view: View,
    hscroll: bool,
    vscroll: bool,
}

#[inherit_methods(from = "self.handle")]
impl ScrollView {
    pub fn new(parent: impl AsWindow) -> Self {
        let mut handle = Widget::new(
            window_class_name(),
            WS_CHILDWINDOW | WS_VISIBLE | WS_VSCROLL | WS_HSCROLL,
            WS_EX_CONTROLPARENT,
            parent.as_window().as_win32(),
        );
        handle.set_size(handle.size_d2l((100, 100)));
        let view = View::new(&handle);
        Self {
            handle,
            view,
            hscroll: true,
            vscroll: true,
        }
    }

    pub fn is_visible(&self) -> bool;

    pub fn set_visible(&mut self, v: bool);

    pub fn is_enabled(&self) -> bool;

    pub fn set_enabled(&mut self, v: bool);

    pub fn loc(&self) -> Point;

    pub fn set_loc(&mut self, p: Point);

    pub fn size(&self) -> Size;

    pub fn set_size(&mut self, v: Size) {
        self.handle.set_size(v);
        let handle = self.handle.as_raw_window().as_win32();
        let view = self.view.as_raw_window().as_win32();

        let mut rect = RECT {
            left: 0,
            top: 0,
            right: 0,
            bottom: 0,
        };
        unsafe extern "system" fn enum_callback(hwnd: HWND, lparam: LPARAM) -> BOOL {
            let mut rect = MaybeUninit::uninit();
            GetWindowRect(hwnd, rect.as_mut_ptr());
            MapWindowPoints(HWND_DESKTOP, GetParent(hwnd), &mut rect as *mut _ as _, 2);
            let rect = rect.assume_init();
            let old_rect = unsafe { &mut *(lparam as *mut RECT) };
            if rect.left < old_rect.left {
                old_rect.left = rect.left;
            }
            if rect.top < old_rect.top {
                old_rect.top = rect.top;
            }
            if rect.right > old_rect.right {
                old_rect.right = rect.right;
            }
            if rect.bottom > old_rect.bottom {
                old_rect.bottom = rect.bottom;
            }
            1
        }
        unsafe {
            EnumChildWindows(view, Some(enum_callback), &mut rect as *mut _ as LPARAM);
        }
        let cwidth = rect.right - rect.left;
        let cheight = rect.bottom - rect.top;
        self.view.set_size(self.handle.size_d2l((cwidth, cheight)));
        let (vwidth, vheight) = self.handle.size_l2d(v);

        let mut si = SCROLLINFO {
            cbSize: size_of::<SCROLLINFO>() as u32,
            fMask: SIF_PAGE | SIF_RANGE,
            nMin: 0,
            nMax: 0,
            nPage: 0,
            nPos: 0,
            nTrackPos: 0,
        };
        let mut x = 0;
        let mut y = 0;
        if self.hscroll {
            si.nPage = vwidth as u32;
            si.nMax = cwidth;
            si.fMask = SIF_PAGE | SIF_RANGE;
            unsafe { SetScrollInfo(handle, SB_HORZ, &si, 1) };
            si.fMask = SIF_POS;
            syscall!(BOOL, GetScrollInfo(handle, SB_HORZ, &mut si)).unwrap();
            x = -si.nPos;
        }
        if self.vscroll {
            si.nPage = vheight as u32;
            si.nMax = cheight;
            si.fMask = SIF_PAGE | SIF_RANGE;
            unsafe { SetScrollInfo(handle, SB_VERT, &si, 1) };
            si.fMask = SIF_POS;
            syscall!(BOOL, GetScrollInfo(handle, SB_VERT, &mut si)).unwrap();
            y = -si.nPos;
        }
        self.scroll_view(x, y);
    }

    pub fn hscroll(&self) -> bool {
        self.hscroll
    }

    pub fn set_hscroll(&mut self, v: bool) {
        self.hscroll = v;
        syscall!(
            BOOL,
            ShowScrollBar(
                self.handle.as_raw_window().as_win32(),
                SB_HORZ,
                if v { 1 } else { 0 }
            )
        )
        .unwrap();
    }

    pub fn vscroll(&self) -> bool {
        self.vscroll
    }

    pub fn set_vscroll(&mut self, v: bool) {
        self.vscroll = v;
        syscall!(
            BOOL,
            ShowScrollBar(
                self.handle.as_raw_window().as_win32(),
                SB_VERT,
                if v { 1 } else { 0 }
            )
        )
        .unwrap();
    }

    fn scroll_view(&self, x: i32, y: i32) {
        syscall!(
            BOOL,
            SetWindowPos(
                self.view.as_raw_window().as_win32(),
                null_mut(),
                x,
                y,
                0,
                0,
                SWP_NOSIZE | SWP_NOZORDER
            )
        )
        .unwrap();
    }

    fn scroll(&self, dir: i32, wparam: WPARAM) {
        let parent = self.handle.as_raw_window().as_win32();
        unsafe {
            let mut si = SCROLLINFO {
                cbSize: size_of::<SCROLLINFO>() as u32,
                fMask: SIF_ALL,
                nMin: 0,
                nMax: 0,
                nPage: 0,
                nPos: 0,
                nTrackPos: 0,
            };
            syscall!(BOOL, GetScrollInfo(parent, dir, &mut si)).unwrap();

            match (wparam & 0xFFFF) as i32 {
                SB_TOP => si.nPos = si.nMin,
                SB_LINEDOWN => si.nPos += 1,
                SB_LINEUP => si.nPos -= 1,
                SB_BOTTOM => si.nPos = si.nMax - si.nPage as i32 + 1,
                SB_PAGEDOWN => si.nPos += si.nPage as i32,
                SB_PAGEUP => si.nPos -= si.nPage as i32,
                SB_THUMBTRACK => si.nPos = si.nTrackPos,
                _ => {}
            }
            si.fMask = SIF_POS;
            SetScrollInfo(parent, dir, &si, 1);
            syscall!(BOOL, GetScrollInfo(parent, dir, &mut si)).unwrap();

            let (x, y) = if dir == SB_HORZ {
                (-si.nPos, 0)
            } else {
                (0, -si.nPos)
            };
            self.scroll_view(x, y);
        }
    }

    pub async fn start(&self) -> ! {
        loop {
            futures_util::select! {
                msg = self.handle.wait(WM_VSCROLL).fuse() => {
                    self.scroll(SB_VERT, msg.wparam);
                },
                msg = self.handle.wait(WM_HSCROLL).fuse() => {
                    self.scroll(SB_HORZ, msg.wparam);
                },
            }
        }
    }
}

winio_handle::impl_as_widget!(ScrollView, handle);
winio_handle::impl_as_window!(ScrollView, view);
