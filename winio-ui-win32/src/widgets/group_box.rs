use std::mem::MaybeUninit;

use inherit_methods_macro::inherit_methods;
use windows_sys::Win32::{
    Foundation::{HWND, LPARAM, LRESULT, WPARAM},
    Graphics::Gdi::{FillRect, HDC},
    UI::{
        Controls::WC_BUTTONW,
        Shell::{DefSubclassProc, RemoveWindowSubclass, SetWindowSubclass},
        WindowsAndMessaging::{
            BS_GROUPBOX, GetClientRect, WM_ERASEBKGND, WS_CHILD, WS_CLIPCHILDREN,
            WS_EX_CONTROLPARENT, WS_VISIBLE,
        },
    },
};
use winio_handle::{AsContainer, AsWidget};
use winio_primitive::{Point, Size};
use winio_ui_windows_common::syscall;

use crate::{Result, View, background_brush, widgets::Widget};

const PADDING: f64 = 3.0;
const GROUP_BOX_SUBCLASS_ID: usize = 0;

#[derive(Debug)]
pub struct GroupBox {
    handle: Widget,
    view: View,
}

#[inherit_methods(from = "self.handle")]
impl GroupBox {
    pub fn new(parent: impl AsContainer) -> Result<Self> {
        let handle = Widget::new(
            WC_BUTTONW,
            WS_CHILD | WS_VISIBLE | WS_CLIPCHILDREN | BS_GROUPBOX as u32,
            WS_EX_CONTROLPARENT,
            parent.as_container().as_win32(),
        )?;
        syscall!(BOOL, unsafe {
            SetWindowSubclass(
                handle.as_widget().as_win32(),
                Some(group_box_wnd_proc),
                GROUP_BOX_SUBCLASS_ID,
                0,
            )
        })?;
        let view = View::new(&handle)?;
        let mut this = Self { handle, view };
        this.update_view()?;
        Ok(this)
    }

    pub fn is_visible(&self) -> Result<bool>;

    pub fn set_visible(&mut self, v: bool) -> Result<()>;

    pub fn loc(&self) -> Result<Point>;

    pub fn set_loc(&mut self, p: Point) -> Result<()>;

    pub fn size(&self) -> Result<Size>;

    pub fn set_size(&mut self, v: Size) -> Result<()> {
        self.handle.set_size(v)?;
        self.update_view()
    }

    pub fn preferred_size(&self) -> Result<Size> {
        self.handle.measure_text()
    }

    pub fn client_size(&self) -> Result<Size> {
        self.view.size()
    }

    pub fn text(&self) -> Result<String>;

    pub fn set_text(&mut self, s: impl AsRef<str>) -> Result<()>;

    fn update_view(&mut self) -> Result<()> {
        let size = self.handle.size()?;
        let title_height = self.handle.measure_text()?.height;
        let top = title_height + PADDING;
        let width = (size.width - PADDING * 2.0).max(0.0);
        let height = (size.height - top - PADDING).max(0.0);
        self.view.set_loc(Point::new(PADDING, top))?;
        self.view.set_size(Size::new(width, height))
    }
}

impl Drop for GroupBox {
    fn drop(&mut self) {
        unsafe {
            RemoveWindowSubclass(
                self.handle.as_widget().as_win32(),
                Some(group_box_wnd_proc),
                GROUP_BOX_SUBCLASS_ID,
            )
        };
    }
}

unsafe extern "system" fn group_box_wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
    _id: usize,
    _data: usize,
) -> LRESULT {
    if msg == WM_ERASEBKGND
        && let Ok(brush) = unsafe { background_brush(hwnd) }
    {
        // A group box only draws its frame and title; the inner area is
        // transparent and relies on its parent to paint the background. The
        // parent usually clips its children, so fill it here.
        let mut rect = MaybeUninit::uninit();
        if unsafe { GetClientRect(hwnd, rect.as_mut_ptr()) } != 0 {
            unsafe { FillRect(wparam as HDC, rect.as_ptr(), brush) };
            return 1;
        }
    }
    unsafe { DefSubclassProc(hwnd, msg, wparam, lparam) }
}

winio_handle::impl_as_widget!(GroupBox, handle);
winio_handle::impl_as_container!(GroupBox, view);
