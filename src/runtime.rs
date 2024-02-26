use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    future::Future,
    mem::MaybeUninit,
    pin::Pin,
    task::{Context, Poll, Waker},
    time::Duration,
};

use compio::driver::AsRawFd;
use compio_log::*;
use slab::Slab;
use windows::Win32::System::Com::{CoInitializeEx, CoUninitialize, COINIT_APARTMENTTHREADED};
use windows_sys::Win32::{
    Foundation::{HANDLE, HWND, LPARAM, LRESULT, POINT, WAIT_FAILED, WPARAM},
    System::Threading::INFINITE,
    UI::WindowsAndMessaging::{
        DefWindowProcW, DispatchMessageW, GetMessagePos, GetMessageTime,
        MsgWaitForMultipleObjectsEx, PeekMessageW, TranslateMessage, MSG, MWMO_ALERTABLE,
        MWMO_INPUTAVAILABLE, PM_REMOVE, QS_ALLINPUT,
    },
};

pub(crate) enum FutureState {
    Active(Option<Waker>),
    Completed(MSG),
}

impl Default for FutureState {
    fn default() -> Self {
        Self::Active(None)
    }
}

struct RegisteredFuture {
    state: FutureState,
    handle: HWND,
    msg: u32,
}

impl RegisteredFuture {
    pub fn new(handle: HWND, msg: u32) -> Self {
        Self {
            state: FutureState::Active(None),
            handle,
            msg,
        }
    }
}

pub struct Runtime {
    runtime: compio::runtime::Runtime,
    registry: RefCell<HashMap<(HWND, u32), HashSet<usize>>>,
    futures: RefCell<Slab<RegisteredFuture>>,
}

impl Runtime {
    pub fn new() -> Self {
        unsafe {
            CoInitializeEx(None, COINIT_APARTMENTTHREADED).unwrap();
        }

        let runtime = compio::runtime::Runtime::new().unwrap();

        Self {
            runtime,
            registry: RefCell::new(HashMap::new()),
            futures: RefCell::new(Slab::new()),
        }
    }

    pub fn block_on<F: Future>(&self, future: F) -> F::Output {
        let _guard = self.runtime.enter();
        let mut result = None;
        unsafe {
            self.runtime
                .spawn_unchecked(async { result = Some(future.await) })
        }
        .detach();
        loop {
            self.runtime.run();
            if let Some(result) = result.take() {
                break result;
            }

            self.runtime.poll_with(Some(Duration::ZERO));

            let timeout = self.runtime.current_timeout();
            let timeout = match timeout {
                Some(timeout) => timeout.as_millis() as u32,
                None => INFINITE,
            };
            let handle = self.runtime.as_raw_fd() as HANDLE;
            trace!("MWMO start");
            let res = unsafe {
                MsgWaitForMultipleObjectsEx(
                    1,
                    &handle,
                    timeout,
                    QS_ALLINPUT,
                    MWMO_ALERTABLE | MWMO_INPUTAVAILABLE,
                )
            };
            trace!("MWMO wake up");
            if res == WAIT_FAILED {
                panic!("{:?}", std::io::Error::last_os_error());
            }

            let mut msg = MaybeUninit::uninit();
            let res = unsafe { PeekMessageW(msg.as_mut_ptr(), 0, 0, 0, PM_REMOVE) };
            if res != 0 {
                let msg = unsafe { msg.assume_init() };
                unsafe {
                    TranslateMessage(&msg);
                    DispatchMessageW(&msg);
                }
            }
        }
    }

    fn set_current_msg(&self, handle: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> bool {
        let pos = unsafe { GetMessagePos() };
        let x = pos as u16;
        let y = (pos >> 16) as u16;
        let msg = MSG {
            hwnd: handle,
            message: msg,
            wParam: wparam,
            lParam: lparam,
            time: unsafe { GetMessageTime() as _ },
            pt: POINT {
                x: x as _,
                y: y as _,
            },
        };
        let completes = self.registry.borrow_mut().remove(&(handle, msg.message));
        if let Some(completes) = completes {
            let dealt = !completes.is_empty();
            let mut futures = self.futures.borrow_mut();
            for id in completes {
                let state = futures.get_mut(id).expect("cannot find registered future");
                let state = std::mem::replace(&mut state.state, FutureState::Completed(msg));
                if let FutureState::Active(Some(w)) = state {
                    w.wake();
                }
            }
            dealt
        } else {
            false
        }
    }

    // Safety: the caller should ensure the handle valid.
    unsafe fn register_message(&self, handle: HWND, msg: u32) -> MsgFuture {
        instrument!(Level::DEBUG, "register_message", ?handle, ?msg);
        let id = self
            .futures
            .borrow_mut()
            .insert(RegisteredFuture::new(handle, msg));
        self.registry
            .borrow_mut()
            .entry((handle, msg))
            .or_default()
            .insert(id);
        debug!("register: {}", id);
        MsgFuture { id }
    }

    fn replace_waker(&self, id: usize, waker: &Waker) -> Option<MSG> {
        let mut futures = self.futures.borrow_mut();
        let state = futures.get_mut(id).expect("cannot find future");
        if let FutureState::Completed(msg) = state.state {
            Some(msg)
        } else {
            state.state = FutureState::Active(Some(waker.clone()));
            None
        }
    }

    fn deregister(&self, id: usize) {
        let state = self.futures.borrow_mut().remove(id);
        if let Some(futures) = self
            .registry
            .borrow_mut()
            .get_mut(&(state.handle, state.msg))
        {
            futures.remove(&id);
        }
    }
}

impl Drop for Runtime {
    fn drop(&mut self) {
        unsafe { CoUninitialize() };
    }
}

thread_local! {
    static RUNTIME: Runtime = Runtime::new();
}

pub fn block_on<F: Future>(future: F) -> F::Output {
    RUNTIME.with(|runtime| runtime.block_on(future))
}

/// # Safety
/// The caller should ensure the handle valid.
pub unsafe fn wait(handle: HWND, msg: u32) -> impl Future<Output = MSG> {
    RUNTIME.with(|runtime| runtime.register_message(handle, msg))
}

pub(crate) unsafe extern "system" fn window_proc(
    handle: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    trace!("window_proc: {}, {}, {}, {}", handle, msg, wparam, lparam);
    let res = RUNTIME.with(|runtime| {
        let res = runtime.set_current_msg(handle, msg, wparam, lparam);
        runtime.runtime.run();
        res
    });
    if res {
        0
    } else {
        DefWindowProcW(handle, msg, wparam, lparam)
    }
}

struct MsgFuture {
    id: usize,
}

impl Future for MsgFuture {
    type Output = MSG;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        instrument!(Level::DEBUG, "MsgFuture", ?self.id);
        if let Some(msg) = RUNTIME.with(|runtime| runtime.replace_waker(self.id, cx.waker())) {
            debug!("ready!");
            Poll::Ready(msg)
        } else {
            debug!("pending...");
            Poll::Pending
        }
    }
}

impl Drop for MsgFuture {
    fn drop(&mut self) {
        RUNTIME.with(|runtime| runtime.deregister(self.id));
    }
}
