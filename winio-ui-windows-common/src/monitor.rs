use std::ptr::{addr_of_mut, null_mut};

use compio::driver::syscall;
use windows_sys::{
    Win32::{
        Foundation::{LPARAM, RECT, S_OK},
        Graphics::Gdi::{EnumDisplayMonitors, GetMonitorInfoW, HDC, HMONITOR, MONITORINFO},
        UI::{
            HiDpi::{GetDpiForMonitor, MDT_EFFECTIVE_DPI},
            WindowsAndMessaging::USER_DEFAULT_SCREEN_DPI,
        },
    },
    core::BOOL,
};
use winio_primitive::{Monitor, Point, Rect, Size};

pub fn monitor_get_all() -> std::io::Result<Vec<Monitor>> {
    let mut res = vec![];
    syscall!(
        BOOL,
        EnumDisplayMonitors(
            null_mut(),
            null_mut(),
            Some(enum_monitor),
            addr_of_mut!(res) as _
        )
    )?;
    Ok(res)
}

unsafe extern "system" fn enum_monitor(m: HMONITOR, _: HDC, _: *mut RECT, res: LPARAM) -> BOOL {
    let res = &mut *(res as *mut Vec<Monitor>);
    let mut info: MONITORINFO = unsafe { std::mem::zeroed() };
    info.cbSize = size_of::<MONITORINFO>() as _;
    if GetMonitorInfoW(m, &mut info) == 0 {
        return 0;
    }
    let mut dpix = 0;
    let mut dpiy = 0;
    if GetDpiForMonitor(m, MDT_EFFECTIVE_DPI, &mut dpix, &mut dpiy) != S_OK {
        return 0;
    }
    res.push(Monitor::new(
        rect_from(info.rcMonitor),
        rect_from(info.rcWork),
        Size::new(dpix as f64, dpiy as f64) / USER_DEFAULT_SCREEN_DPI as f64,
    ));
    1
}

#[inline]
fn rect_from(r: RECT) -> Rect {
    Rect::new(
        Point::new(r.left as f64, r.top as f64),
        Size::new((r.right - r.left) as f64, (r.bottom - r.top) as f64),
    )
}
