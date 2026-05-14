#[cfg(feature = "once_cell_try")]
use std::cell::OnceCell;
use std::{
    os::windows::io::{AsRawHandle, BorrowedHandle, OwnedHandle},
    sync::Arc,
    task::{Wake, Waker},
    time::Duration,
};

use compio::driver::RawFd;
#[cfg(not(feature = "once_cell_try"))]
use once_cell::sync::OnceCell;
use windows::Win32::Graphics::Direct2D::{
    D2D1_FACTORY_TYPE_SINGLE_THREADED, D2D1CreateFactory, ID2D1Factory2,
};
use windows_sys::Win32::{
    Foundation::{HWND, WAIT_FAILED, WAIT_OBJECT_0},
    System::Threading::{GetCurrentThread, INFINITE, QueueUserAPC},
    UI::WindowsAndMessaging::{
        MSG, MWMO_ALERTABLE, MWMO_INPUTAVAILABLE, MsgWaitForMultipleObjectsEx, PM_REMOVE,
        PeekMessageW, QS_ALLINPUT, WM_QUIT,
    },
};

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
        let mut timeout = if TIMEOUT.is_set() {
            TIMEOUT.with(|t| *t)
        } else {
            None
        };
        let is_ready = winio_pollable::run_current_task();
        if is_ready {
            timeout = Some(Duration::from_millis(0));
        }
        let timeout = match timeout {
            Some(timeout) => timeout.as_millis() as u32,
            None => INFINITE,
        };
        let (res, queue_res) = if HANDLE.is_set() {
            let handle = HANDLE.with(|h| *h);
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
            (res, WAIT_OBJECT_1)
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
            (res, WAIT_OBJECT_0)
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
            _ => {}
        }
    }
}

scoped_tls::scoped_thread_local!(static TIMEOUT: Option<Duration>);

scoped_tls::scoped_thread_local!(static HANDLE: RawFd);

struct ApcWaker {
    handle: OwnedHandle,
}

impl ApcWaker {
    pub fn new() -> std::io::Result<Self> {
        let handle = unsafe { GetCurrentThread() };
        let handle = unsafe { BorrowedHandle::borrow_raw(handle) }.try_clone_to_owned()?;
        Ok(Self { handle })
    }

    fn wake_impl(&self) {
        unsafe {
            QueueUserAPC(Some(Self::apc_proc), self.handle.as_raw_handle() as _, 0);
        }
    }

    unsafe extern "system" fn apc_proc(_: usize) {}
}

impl Wake for ApcWaker {
    fn wake(self: Arc<Self>) {
        self.wake_impl();
    }

    fn wake_by_ref(self: &Arc<Self>) {
        self.wake_impl();
    }
}

pub fn waker() -> std::io::Result<Waker> {
    Ok(Waker::from(Arc::new(ApcWaker::new()?)))
}
