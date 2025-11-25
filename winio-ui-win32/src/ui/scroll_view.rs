use std::{
    io,
    mem::{MaybeUninit, size_of},
    ptr::null_mut,
};

use compio::driver::syscall;
use compio_log::error;
use futures_util::FutureExt;
use inherit_methods_macro::inherit_methods;
use windows_sys::{
    Win32::{
        Foundation::{GetLastError, HWND, LPARAM, RECT, SetLastError, WPARAM},
        Graphics::Gdi::MapWindowPoints,
        UI::{
            Controls::{SetScrollInfo, ShowScrollBar},
            WindowsAndMessaging::{
                EnumChildWindows, GetParent, GetScrollInfo, GetWindowRect, HWND_DESKTOP, SB_BOTTOM,
                SB_HORZ, SB_LINEDOWN, SB_LINEUP, SB_PAGEDOWN, SB_PAGEUP, SB_THUMBTRACK, SB_TOP,
                SB_VERT, SCROLLINFO, SIF_ALL, SIF_PAGE, SIF_POS, SIF_RANGE, SWP_NOSIZE,
                SWP_NOZORDER, SetWindowPos, WM_HSCROLL, WM_MOUSEHWHEEL, WM_MOUSEWHEEL, WM_VSCROLL,
                WS_CHILDWINDOW, WS_CLIPCHILDREN, WS_EX_CONTROLPARENT, WS_HSCROLL, WS_VISIBLE,
                WS_VSCROLL,
            },
        },
    },
    core::BOOL,
};
use winio_handle::{AsContainer, AsRawWidget};
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
    pub fn new(parent: impl AsContainer) -> io::Result<Self> {
        let handle = Widget::new(
            window_class_name(),
            WS_CHILDWINDOW | WS_CLIPCHILDREN | WS_VISIBLE | WS_VSCROLL | WS_HSCROLL,
            WS_EX_CONTROLPARENT,
            parent.as_container().as_win32(),
        )?;
        let view = View::new(&handle)?;
        Ok(Self {
            handle,
            view,
            hscroll: true,
            vscroll: true,
        })
    }

    pub fn is_visible(&self) -> io::Result<bool>;

    pub fn set_visible(&mut self, v: bool) -> io::Result<()>;

    pub fn is_enabled(&self) -> io::Result<bool>;

    pub fn set_enabled(&mut self, v: bool) -> io::Result<()>;

    pub fn loc(&self) -> io::Result<Point>;

    pub fn set_loc(&mut self, p: Point) -> io::Result<()>;

    pub fn size(&self) -> io::Result<Size>;

    pub fn set_size(&mut self, v: Size) -> io::Result<()> {
        self.handle.set_size(v)?;
        let handle = self.handle.as_raw_widget().as_win32();
        let view = self.view.as_raw_widget().as_win32();

        let mut rect = RECT {
            left: 0,
            top: 0,
            right: 0,
            bottom: 0,
        };
        unsafe extern "system" fn enum_callback(hwnd: HWND, lparam: LPARAM) -> BOOL {
            let mut rect = MaybeUninit::uninit();
            if GetWindowRect(hwnd, rect.as_mut_ptr()) == 0 {
                return 0;
            }
            SetLastError(0);
            if MapWindowPoints(HWND_DESKTOP, GetParent(hwnd), &mut rect as *mut _ as _, 2) == 0
                && GetLastError() != 0
            {
                return 0;
            }
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
        let (vwidth, vheight) = self.handle.size_l2d(v);
        let cwidth = (rect.right - rect.left).max(vwidth - 2);
        let cheight = (rect.bottom - rect.top).max(vheight - 2);
        self.view
            .set_size(self.handle.size_d2l((cwidth, cheight)))?;

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
            syscall!(BOOL, GetScrollInfo(handle, SB_HORZ, &mut si))?;
            x = -si.nPos;
        }
        if self.vscroll {
            si.nPage = vheight as u32;
            si.nMax = cheight;
            si.fMask = SIF_PAGE | SIF_RANGE;
            unsafe { SetScrollInfo(handle, SB_VERT, &si, 1) };
            si.fMask = SIF_POS;
            syscall!(BOOL, GetScrollInfo(handle, SB_VERT, &mut si))?;
            y = -si.nPos;
        }
        self.scroll_view(x, y)
    }

    pub fn hscroll(&self) -> io::Result<bool> {
        Ok(self.hscroll)
    }

    pub fn set_hscroll(&mut self, v: bool) -> io::Result<()> {
        self.hscroll = v;
        syscall!(
            BOOL,
            ShowScrollBar(
                self.handle.as_raw_widget().as_win32(),
                SB_HORZ,
                if v { 1 } else { 0 }
            )
        )?;
        Ok(())
    }

    pub fn vscroll(&self) -> io::Result<bool> {
        Ok(self.vscroll)
    }

    pub fn set_vscroll(&mut self, v: bool) -> io::Result<()> {
        self.vscroll = v;
        syscall!(
            BOOL,
            ShowScrollBar(
                self.handle.as_raw_widget().as_win32(),
                SB_VERT,
                if v { 1 } else { 0 }
            )
        )?;
        Ok(())
    }

    fn scroll_view(&self, x: i32, y: i32) -> io::Result<()> {
        syscall!(
            BOOL,
            SetWindowPos(
                self.view.as_raw_widget().as_win32(),
                null_mut(),
                x,
                y,
                0,
                0,
                SWP_NOSIZE | SWP_NOZORDER
            )
        )?;
        Ok(())
    }

    fn scroll(&self, dir: i32, wparam: WPARAM, wheel: bool) -> io::Result<()> {
        let parent = self.handle.as_raw_widget().as_win32();
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
            syscall!(BOOL, GetScrollInfo(parent, dir, &mut si))?;

            if wheel {
                let delta = ((wparam >> 16) & 0xFFFF) as i16 as isize;
                si.nPos += -delta as i32;
            } else {
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
            }
            si.fMask = SIF_POS;
            SetScrollInfo(parent, dir, &si, 1);
            syscall!(BOOL, GetScrollInfo(parent, dir, &mut si))?;

            let (x, y) = if dir == SB_HORZ {
                (-si.nPos, 0)
            } else {
                (0, -si.nPos)
            };
            self.scroll_view(x, y)
        }
    }

    pub async fn start(&self) -> ! {
        loop {
            let res = futures_util::select! {
                msg = self.handle.wait(WM_VSCROLL).fuse() => {
                    self.scroll(SB_VERT, msg.wparam(), false)
                },
                msg = self.handle.wait(WM_HSCROLL).fuse() => {
                    self.scroll(SB_HORZ, msg.wparam(), false)
                },
                msg = self.handle.wait(WM_MOUSEWHEEL).fuse() => {
                    self.scroll(SB_VERT, msg.wparam(), true)
                },
                msg = self.handle.wait(WM_MOUSEHWHEEL).fuse() => {
                    self.scroll(SB_HORZ, msg.wparam(), true)
                },
            };
            match res {
                Ok(()) => {}
                Err(_e) => {
                    error!("scroll error: {_e:?}");
                }
            }
        }
    }
}

winio_handle::impl_as_widget!(ScrollView, handle);
winio_handle::impl_as_container!(ScrollView, view);
