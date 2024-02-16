use std::{
    cell::{LazyCell, RefCell},
    collections::{HashMap, HashSet},
    future::Future,
    io,
    mem::MaybeUninit,
    pin::Pin,
    ptr::null,
    sync::Arc,
    task::{Context, Poll, Waker},
};

use async_task::{Runnable, Task};
use compio_log::*;
use crossbeam_queue::SegQueue;
use slab::Slab;
use windows_sys::Win32::{
    Foundation::{HWND, LPARAM, LRESULT, POINT, WAIT_FAILED, WPARAM},
    Networking::WinSock::{WSACleanup, WSAStartup, WSADATA},
    System::{
        Com::{CoInitializeEx, CoUninitialize, COINIT_APARTMENTTHREADED},
        Threading::INFINITE,
    },
    UI::WindowsAndMessaging::{
        DefWindowProcW, DispatchMessageW, GetMessagePos, GetMessageTime,
        MsgWaitForMultipleObjectsEx, PeekMessageW, TranslateMessage, MSG, MWMO_ALERTABLE,
        MWMO_INPUTAVAILABLE, PM_REMOVE, QS_ALLINPUT,
    },
};

use crate::{syscall_hresult, syscall_socket};

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
    runnables: Arc<SegQueue<Runnable>>,
    registry: RefCell<HashMap<(HWND, u32), HashSet<usize>>>,
    futures: RefCell<Slab<RegisteredFuture>>,
}

impl Runtime {
    pub fn new() -> Self {
        let mut data: WSADATA = unsafe { std::mem::zeroed() };
        syscall_socket(unsafe { WSAStartup(0x202, &mut data) }).unwrap();

        syscall_hresult(unsafe { CoInitializeEx(null(), COINIT_APARTMENTTHREADED) }).unwrap();

        Self {
            runnables: Arc::new(SegQueue::new()),
            registry: RefCell::new(HashMap::new()),
            futures: RefCell::new(Slab::new()),
        }
    }

    // Safety: the caller ensures lifetimes.
    unsafe fn spawn_unchecked<F: Future>(&self, future: F) -> Task<F::Output> {
        let runnables = self.runnables.clone();
        let schedule = move |runnable| {
            runnables.push(runnable);
        };
        let (runnable, task) = async_task::spawn_unchecked(future, schedule);
        runnable.schedule();
        task
    }

    pub fn block_on<F: Future>(&self, future: F) -> F::Output {
        let mut result = None;
        unsafe { self.spawn_unchecked(async { result = Some(future.await) }) }.detach();
        loop {
            self.run_tasks();
            if let Some(result) = result.take() {
                return result;
            }
            poll_thread().expect("failed to poll message queue");
        }
    }

    pub fn spawn<F: Future + 'static>(&self, future: F) -> Task<F::Output> {
        unsafe { self.spawn_unchecked(future) }
    }

    fn run_tasks(&self) {
        loop {
            let next_task = self.runnables.pop();
            if let Some(task) = next_task {
                task.run();
            } else {
                break;
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

        syscall_socket(unsafe { WSACleanup() }).unwrap();
    }
}

#[thread_local]
static RUNTIME: LazyCell<Runtime> = LazyCell::new(Runtime::new);

fn poll_thread() -> io::Result<()> {
    trace!("MWMO start");
    let res = unsafe {
        MsgWaitForMultipleObjectsEx(
            0,
            null(),
            INFINITE,
            QS_ALLINPUT,
            MWMO_INPUTAVAILABLE | MWMO_ALERTABLE,
        )
    };
    if res == WAIT_FAILED {
        return Err(io::Error::last_os_error());
    }
    trace!("MWMO wake up");
    let mut msg = MaybeUninit::uninit();
    let res = unsafe { PeekMessageW(msg.as_mut_ptr(), 0, 0, 0, PM_REMOVE) };
    if res != 0 {
        // Safety: message arrives
        let msg = unsafe { msg.assume_init() };
        unsafe {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
    Ok(())
}

pub fn block_on<F: Future>(future: F) -> F::Output {
    RUNTIME.block_on(future)
}

pub fn spawn<F: Future + 'static>(future: F) -> Task<F::Output> {
    RUNTIME.spawn(future)
}

/// # Safety
/// The caller should ensure the handle valid.
pub unsafe fn wait(handle: HWND, msg: u32) -> impl Future<Output = MSG> {
    RUNTIME.register_message(handle, msg)
}

pub(crate) unsafe extern "system" fn window_proc(
    handle: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    trace!("window_proc: {}, {}, {}, {}", handle, msg, wparam, lparam);
    let res = RUNTIME.set_current_msg(handle, msg, wparam, lparam);
    RUNTIME.run_tasks();
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
        if let Some(msg) = RUNTIME.replace_waker(self.id, cx.waker()) {
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
        RUNTIME.deregister(self.id);
    }
}
