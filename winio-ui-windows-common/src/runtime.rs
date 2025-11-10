use std::{cell::OnceCell, ops::Deref, time::Duration};

use compio::driver::AsRawFd;
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
    runtime: compio::runtime::Runtime,
    d2d1: OnceCell<ID2D1Factory2>,
}

impl Runtime {
    pub fn new() -> std::io::Result<Self> {
        let runtime = compio::runtime::Runtime::new()?;
        Ok(Self {
            runtime,
            d2d1: OnceCell::new(),
        })
    }

    pub fn d2d1(&self) -> &ID2D1Factory2 {
        self.d2d1.get_or_init(|| unsafe {
            D2D1CreateFactory(D2D1_FACTORY_TYPE_SINGLE_THREADED, None).unwrap()
        })
    }

    pub fn run(&self) -> bool {
        self.runtime.run()
    }

    /// # Safety
    /// This function calls [`PeekMessageW`] internally.
    pub unsafe fn get_message(&self, msg: *mut MSG, hwnd: HWND, min: u32, max: u32) -> Option<i32> {
        self.runtime.poll_with(Some(Duration::ZERO));
        let remaining_tasks = self.run();
        let timeout = if remaining_tasks {
            Some(Duration::ZERO)
        } else {
            self.runtime.current_timeout()
        };
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
                        Some(0)
                    } else {
                        Some(1)
                    }
                } else {
                    None
                }
            }
            WAIT_FAILED => Some(-1),
            _ => None,
        }
    }
}

impl Deref for Runtime {
    type Target = compio::runtime::Runtime;

    fn deref(&self) -> &Self::Target {
        &self.runtime
    }
}
