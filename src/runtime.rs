use std::{
    cell::{LazyCell, RefCell},
    collections::HashMap,
    future::{ready, Future},
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
use futures_util::future::Either;
use slab::Slab;
use windows_sys::Win32::{
    Foundation::{HWND, LPARAM, LRESULT, POINT, WAIT_FAILED, WPARAM},
    System::Threading::INFINITE,
    UI::WindowsAndMessaging::{
        DefWindowProcW, DispatchMessageW, GetMessagePos, GetMessageTime,
        MsgWaitForMultipleObjectsEx, PeekMessageW, TranslateMessage, MSG, MWMO_ALERTABLE,
        MWMO_INPUTAVAILABLE, PM_REMOVE, QS_ALLINPUT,
    },
};

pub(crate) enum FutureState {
    Active(Option<Waker>),
    Completed,
}

impl Default for FutureState {
    fn default() -> Self {
        Self::Active(None)
    }
}

pub struct Runtime {
    runnables: Arc<SegQueue<Runnable>>,
    current_msg: RefCell<MSG>,
    registry: RefCell<HashMap<(HWND, u32), Vec<usize>>>,
    futures: RefCell<Slab<FutureState>>,
}

impl Runtime {
    pub fn new() -> Self {
        Self {
            runnables: Arc::new(SegQueue::new()),
            current_msg: unsafe { std::mem::zeroed() },
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
        self.run_tasks();
        loop {
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
        let x = (pos & 0xFFFF) as _;
        let y = (pos >> 16) as _;
        *self.current_msg.borrow_mut() = MSG {
            hwnd: handle,
            message: msg,
            wParam: wparam,
            lParam: lparam,
            time: unsafe { GetMessageTime() as _ },
            pt: POINT { x, y },
        };
        let completes = self.registry.borrow_mut().remove(&(handle, msg));
        if let Some(completes) = completes {
            let dealt = !completes.is_empty();
            let mut futures = self.futures.borrow_mut();
            for id in completes {
                let state = futures.get_mut(id).expect("cannot find registered future");
                let state = std::mem::replace(state, FutureState::Completed);
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
    unsafe fn register_message(&self, handle: HWND, msg: u32) -> Option<MsgFuture> {
        instrument!(Level::DEBUG, "register_message", ?handle, ?msg);
        let curr_msg = self.current_msg.borrow();
        if curr_msg.hwnd == handle && curr_msg.message == msg {
            debug!("ready");
            None
        } else {
            let id = self.futures.borrow_mut().insert(FutureState::Active(None));
            self.registry
                .borrow_mut()
                .entry((handle, msg))
                .or_default()
                .push(id);
            debug!("register: {}", id);
            Some(MsgFuture { id })
        }
    }

    fn replace_waker(&self, id: usize, waker: &Waker) -> bool {
        let mut futures = self.futures.borrow_mut();
        let state = futures.get_mut(id).expect("cannot find future");
        if let FutureState::Completed = state {
            true
        } else {
            *state = FutureState::Active(Some(waker.clone()));
            false
        }
    }
}

#[thread_local]
static RUNTIME: LazyCell<Runtime> = LazyCell::new(Runtime::new);

fn poll_thread() -> io::Result<()> {
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
    if let Some(future) = RUNTIME.register_message(handle, msg) {
        Either::Left(future)
    } else {
        Either::Right(ready(*RUNTIME.current_msg.borrow()))
    }
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
        if RUNTIME.replace_waker(self.id, cx.waker()) {
            debug!("ready!");
            Poll::Ready(*RUNTIME.current_msg.borrow())
        } else {
            debug!("pending...");
            Poll::Pending
        }
    }
}
