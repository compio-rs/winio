use std::{cell::RefCell, collections::HashMap, ptr::null_mut, rc::Rc, time::Duration};

use windows_sys::Win32::{
    Foundation::HWND,
    UI::WindowsAndMessaging::{KillTimer, SetTimer},
};
use winio_callback::Callback;
use winio_pollable::GlobalRuntime;

use crate::{Result, syscall};

thread_local! {
    static TIMER_MAP: RefCell<HashMap<usize, Rc<Callback>>> = RefCell::new(HashMap::new());
}

#[derive(Debug)]
pub struct Timer {
    duration: Duration,
    id: usize,
    callback: Rc<Callback>,
}

impl Timer {
    pub fn new(duration: Duration) -> Result<Self> {
        Ok(Self {
            duration,
            id: 0,
            callback: Rc::new(Callback::new()),
        })
    }

    pub fn start(&mut self) -> Result<()> {
        self.stop()?;
        let id = syscall!(
            BOOL,
            SetTimer(
                null_mut(),
                0,
                self.duration.as_millis() as u32,
                Some(timer_handler),
            )
        )?;
        self.id = id;
        TIMER_MAP.with_borrow_mut(|map| map.insert(id, self.callback.clone()));
        Ok(())
    }

    pub fn stop(&mut self) -> Result<()> {
        let id = self.id;
        self.id = 0;
        if id != 0 {
            syscall!(BOOL, KillTimer(null_mut(), id))?;
            TIMER_MAP.with_borrow_mut(|map| map.remove(&id));
        }
        Ok(())
    }

    pub fn is_enabled(&self) -> Result<bool> {
        Ok(self.id != 0)
    }

    pub async fn wait(&self) {
        self.callback.wait().await
    }
}

unsafe extern "system" fn timer_handler(_hwnd: HWND, _umsg: u32, idevent: usize, _dwtime: u32) {
    TIMER_MAP.with_borrow(|map| {
        if let Some(callback) = map.get(&idevent) {
            callback.signal::<GlobalRuntime>(());
        }
    })
}
