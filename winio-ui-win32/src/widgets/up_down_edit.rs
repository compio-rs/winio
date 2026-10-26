use inherit_methods_macro::inherit_methods;
use windows::Win32::Foundation::E_INVALIDARG;
use windows_sys::Win32::{
    Foundation::LPARAM,
    UI::{
        Controls::{
            UDACCEL, UDM_GETACCEL, UDM_GETPOS32, UDM_GETRANGE32, UDM_SETACCEL, UDM_SETBUDDY,
            UDM_SETPOS32, UDM_SETRANGE32, UDS_ARROWKEYS, UDS_HOTTRACK, UDS_NOTHOUSANDS,
            UDS_SETBUDDYINT, UPDOWN_CLASSW, WC_EDITW,
        },
        HiDpi::GetSystemMetricsForDpi,
        WindowsAndMessaging::{
            EN_CHANGE, EN_UPDATE, ES_AUTOHSCROLL, ES_LEFT, ES_NUMBER, SM_CXVSCROLL, SM_CYVSCROLL,
            WM_COMMAND, WS_CHILD, WS_CLIPCHILDREN, WS_EX_CLIENTEDGE, WS_TABSTOP, WS_VISIBLE,
        },
    },
};
use winio_handle::{AsContainer, AsWidget};
use winio_primitive::{Point, Size};

use crate::{Error, Result, Widget};

#[derive(Debug)]
pub struct UpDownEdit {
    updown: Widget,
    handle: Widget,
}

#[inherit_methods(from = "self.handle")]
impl UpDownEdit {
    pub fn new(parent: impl AsContainer) -> Result<Self> {
        let parent = parent.as_container().as_win32();
        let handle = Widget::new(
            WC_EDITW,
            WS_CHILD
                | WS_VISIBLE
                | WS_TABSTOP
                | WS_CLIPCHILDREN
                | ES_LEFT as u32
                | ES_AUTOHSCROLL as u32
                | ES_NUMBER as u32,
            WS_EX_CLIENTEDGE,
            parent,
        )?;
        let updown = Widget::new(
            UPDOWN_CLASSW,
            WS_CHILD
                | WS_VISIBLE
                | UDS_SETBUDDYINT
                | UDS_NOTHOUSANDS
                | UDS_ARROWKEYS
                | UDS_HOTTRACK,
            0,
            handle.as_widget().as_win32(),
        )?;
        updown.send_message(UDM_SETBUDDY, handle.as_widget().as_win32() as _, 0);
        let width = unsafe { GetSystemMetricsForDpi(SM_CXVSCROLL, 96) };
        let height = unsafe { GetSystemMetricsForDpi(SM_CYVSCROLL, 96) };
        updown.set_size(Size::new(width as _, height as _))?;

        Ok(Self { updown, handle })
    }

    pub fn is_visible(&self) -> Result<bool>;

    pub fn set_visible(&mut self, v: bool) -> Result<()>;

    pub fn is_enabled(&self) -> Result<bool>;

    pub fn set_enabled(&mut self, v: bool) -> Result<()>;

    pub fn preferred_size(&self) -> Result<Size> {
        let edit = self.handle.measure_text()?;
        let updown = self.updown.size()?;
        Ok(Size::new(
            edit.width + updown.width + 8.0,
            (edit.height + 4.0).max(updown.height),
        ))
    }

    pub fn loc(&self) -> Result<Point>;

    pub fn set_loc(&mut self, p: Point) -> Result<()>;

    pub fn size(&self) -> Result<Size>;

    pub fn set_size(&mut self, s: Size) -> Result<()> {
        self.handle.set_size(s)?;
        let updown_size = self.updown.size()?;
        self.updown
            .set_loc(Point::new(s.width - self.updown.size()?.width, 0.0))?;
        self.updown
            .set_size(Size::new(updown_size.width, s.height))?;
        Ok(())
    }

    pub fn tooltip(&self) -> Result<String>;

    pub fn set_tooltip(&mut self, s: impl AsRef<str>) -> Result<()>;

    pub fn minimum(&self) -> Result<usize> {
        let mut min = 0i32;
        self.updown
            .send_message(UDM_GETRANGE32, &raw mut min as _, 0);
        Ok(min as usize)
    }

    pub fn set_minimum(&mut self, v: usize) -> Result<()> {
        let max = self.maximum()?.max(v + 1);
        self.updown.send_message(UDM_SETRANGE32, v as _, max as _);
        Ok(())
    }

    pub fn maximum(&self) -> Result<usize> {
        let mut max = 0i32;
        self.updown
            .send_message(UDM_GETRANGE32, 0, &raw mut max as _);
        Ok(max as usize)
    }

    pub fn set_maximum(&mut self, v: usize) -> Result<()> {
        let min = self.minimum()?;
        let v = v.max(min + 1);
        self.updown.send_message(UDM_SETRANGE32, min as _, v as _);
        Ok(())
    }

    pub fn step(&self) -> Result<usize> {
        let mut accel: UDACCEL = unsafe { std::mem::zeroed() };
        let res = self
            .updown
            .send_message(UDM_GETACCEL, 1, &raw mut accel as _);
        if res < 1 {
            Ok(1)
        } else {
            Ok(accel.nInc as usize)
        }
    }

    pub fn set_step(&mut self, v: usize) -> Result<()> {
        let accel = UDACCEL {
            nSec: 0,
            nInc: v as u32,
        };
        self.updown
            .send_message(UDM_SETACCEL, 1, &raw const accel as _);
        Ok(())
    }

    pub fn value(&self) -> Result<usize> {
        let mut err = 0i32;
        let res = self.updown.send_message(UDM_GETPOS32, 0, &raw mut err as _);
        if err != 0 {
            Err(Error::from(E_INVALIDARG))
        } else {
            Ok(res as usize)
        }
    }

    pub fn set_value(&mut self, v: usize) -> Result<()> {
        self.updown.send_message(UDM_SETPOS32, 0, v as LPARAM);
        Ok(())
    }

    pub async fn wait_change(&self) {
        loop {
            let msg = self.handle.wait_parent(WM_COMMAND).await.command();
            if std::ptr::eq(msg.handle, self.handle.as_widget().as_win32())
                && (msg.message == EN_CHANGE || msg.message == EN_UPDATE)
                && self.value().is_ok()
            {
                break;
            }
        }
    }
}

winio_handle::impl_as_widget!(UpDownEdit, handle);
