#[cfg(feature = "once_cell_try")]
use std::cell::OnceCell;
use std::ops::Deref;

use compio::driver::AsRawFd;
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

pub struct Runtime {
    runtime: winio_pollable::Runtime,
    d2d1: OnceCell<ID2D1Factory2>,
}

impl Runtime {
    pub fn new() -> crate::Result<Self> {
        let runtime = winio_pollable::Runtime::new()?;
        Ok(Self {
            runtime,
            d2d1: OnceCell::new(),
        })
    }

    pub fn d2d1(&self) -> crate::Result<&ID2D1Factory2> {
        self.d2d1.get_or_try_init(|| unsafe {
            D2D1CreateFactory(D2D1_FACTORY_TYPE_SINGLE_THREADED, None)
        })
    }

    pub fn run(&self) -> bool {
        self.runtime.run()
    }

    /// # Safety
    /// This function calls [`PeekMessageW`] internally.
    pub unsafe fn get_message(&self, msg: *mut MSG, hwnd: HWND, min: u32, max: u32) -> i32 {
        loop {
            let timeout = self.runtime.poll_and_run();
            let timeout = match timeout {
                Some(timeout) => timeout.as_millis() as u32,
                None => INFINITE,
            };
            let handle = self.runtime.as_raw_fd();
            let res = MsgWaitForMultipleObjectsEx(
                1,
                &handle,
                timeout,
                QS_ALLINPUT,
                MWMO_ALERTABLE | MWMO_INPUTAVAILABLE,
            );
            const WAIT_OBJECT_1: u32 = WAIT_OBJECT_0 + 1;
            match res {
                WAIT_OBJECT_1 => {
                    let res = PeekMessageW(msg, hwnd, min, max, PM_REMOVE);
                    if res != 0 {
                        if (*msg).message == WM_QUIT {
                            return 0;
                        } else {
                            return 1;
                        }
                    }
                }
                WAIT_FAILED => return -1,
                _ => {}
            }
        }
    }
}

impl Deref for Runtime {
    type Target = winio_pollable::Runtime;

    fn deref(&self) -> &Self::Target {
        &self.runtime
    }
}
