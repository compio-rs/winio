#[cfg(feature = "once_cell_try")]
use std::cell::OnceCell;
use std::time::Duration;

#[cfg(not(feature = "once_cell_try"))]
use once_cell::sync::OnceCell;
use windows::Win32::Graphics::Direct2D::{
    D2D1_FACTORY_TYPE_SINGLE_THREADED, D2D1CreateFactory, ID2D1Factory2,
};
use windows_sys::Win32::{
    Foundation::{HWND, WAIT_FAILED, WAIT_OBJECT_0},
    System::Threading::INFINITE,
    UI::WindowsAndMessaging::{
        MSG, MWMO_ALERTABLE, MWMO_INPUTAVAILABLE, MsgWaitForMultipleObjectsEx, PM_REMOVE,
        PeekMessageW, QS_ALLINPUT, WM_QUIT,
    },
};
#[cfg(not(feature = "compio-compat"))]
use {std::task::Waker, windows_sys::Win32::Foundation::HANDLE};

#[cfg(feature = "compio-compat")]
use crate::get_handle;

#[cfg(not(feature = "compio-compat"))]
const fn get_handle() -> (Option<HANDLE>, Option<Duration>, Option<Waker>) {
    (None, None, None)
}

thread_local! {
    static D2D1_FACTORY: OnceCell<ID2D1Factory2> = const { OnceCell::new() };
}

pub fn d2d1_factory() -> crate::Result<ID2D1Factory2> {
    D2D1_FACTORY.with(|d2d1| {
        d2d1.get_or_try_init(|| unsafe {
            D2D1CreateFactory(D2D1_FACTORY_TYPE_SINGLE_THREADED, None)
        })
        .cloned()
    })
}

/// # Safety
/// This function calls [`PeekMessageW`] internally.
pub unsafe fn get_message(msg: *mut MSG, hwnd: HWND, min: u32, max: u32) -> i32 {
    loop {
        let (handle, mut timeout, waker) = get_handle();
        let is_ready = winio_pollable::run_current_task();
        if is_ready {
            timeout = Some(Duration::from_millis(0));
        }
        let timeout = match timeout {
            Some(timeout) => timeout.as_millis() as u32,
            None => INFINITE,
        };
        let (res, queue_res, handle_res) = if let Some(handle) = handle {
            let res = unsafe {
                MsgWaitForMultipleObjectsEx(
                    1,
                    &handle,
                    timeout,
                    QS_ALLINPUT,
                    MWMO_ALERTABLE | MWMO_INPUTAVAILABLE,
                )
            };
            const WAIT_OBJECT_1: u32 = WAIT_OBJECT_0 + 1;
            (res, WAIT_OBJECT_1, Some(WAIT_OBJECT_0))
        } else {
            let res = unsafe {
                MsgWaitForMultipleObjectsEx(
                    0,
                    std::ptr::null(),
                    timeout,
                    QS_ALLINPUT,
                    MWMO_ALERTABLE | MWMO_INPUTAVAILABLE,
                )
            };
            (res, WAIT_OBJECT_0, None)
        };
        match res {
            WAIT_FAILED => return -1,
            res if res == queue_res => {
                let res = unsafe { PeekMessageW(msg, hwnd, min, max, PM_REMOVE) };
                if res != 0 {
                    if unsafe { (*msg).message } == WM_QUIT {
                        return 0;
                    } else {
                        return 1;
                    }
                }
            }
            res if Some(res) == handle_res
                && let Some(waker) = waker =>
            {
                waker.wake();
            }
            _ => {}
        }
    }
}
