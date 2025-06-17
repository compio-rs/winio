use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    future::Future,
    mem::MaybeUninit,
    pin::Pin,
    ptr::{null, null_mut},
    sync::LazyLock,
    task::{Context, Poll, Waker},
    time::Duration,
};

use compio::driver::AsRawFd;
use compio_log::*;
use slab::Slab;
use windows::Win32::System::Com::{COINIT_MULTITHREADED, CoInitializeEx, CoUninitialize};
use windows_sys::{
    Win32::{
        Foundation::{COLORREF, HANDLE, HWND, LPARAM, LRESULT, POINT, RECT, WAIT_FAILED, WPARAM},
        Graphics::Gdi::{
            BLACK_BRUSH, CreateSolidBrush, GetStockObject, HDC, InvalidateRect, ScreenToClient,
            SetBkColor, SetTextColor,
        },
        System::Threading::INFINITE,
        UI::{
            Controls::DRAWITEMSTRUCT,
            WindowsAndMessaging::{
                ChildWindowFromPoint, DefWindowProcW, DispatchMessageW, EnumChildWindows,
                GetCursorPos, GetMessagePos, GetMessageTime, MSG, MWMO_ALERTABLE,
                MWMO_INPUTAVAILABLE, MsgWaitForMultipleObjectsEx, PM_REMOVE, PeekMessageW,
                QS_ALLINPUT, SWP_NOACTIVATE, SWP_NOZORDER, SendMessageW, SetWindowPos,
                TranslateMessage, WM_COMMAND, WM_CREATE, WM_CTLCOLORBTN, WM_CTLCOLOREDIT,
                WM_CTLCOLORLISTBOX, WM_CTLCOLORSTATIC, WM_DPICHANGED, WM_DRAWITEM, WM_SETFONT,
                WM_SETTINGCHANGE,
            },
        },
    },
    core::BOOL,
};

use super::RUNTIME;
use crate::ui::{
    darkmode::{
        children_refresh_dark_mode, control_color_static, init_dark, is_dark_mode_allowed_for_app,
        window_use_dark_mode,
    },
    dpi::get_dpi_for_window,
    font::{WinBrush, default_font},
};

#[derive(Clone, Copy)]
pub(crate) struct WindowMessage {
    pub message: MSG,
    pub detail: Option<WindowMessageDetail>,
}

#[derive(Clone, Copy)]
#[non_exhaustive]
pub(crate) enum WindowMessageDetail {
    #[allow(dead_code)]
    Command {
        message: u32,
        id: u32,
        handle: HWND,
    },
    DrawItem(DRAWITEMSTRUCT),
}

pub(crate) enum FutureState {
    Active(Option<Waker>),
    Completed(WindowMessage),
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
            init_dark();
            CoInitializeEx(None, COINIT_MULTITHREADED).unwrap();
        }

        let runtime = compio::runtime::Runtime::new().unwrap();

        Self {
            runtime,
            registry: RefCell::new(HashMap::new()),
            futures: RefCell::new(Slab::new()),
        }
    }

    fn enter<T, F: FnOnce() -> T>(&self, f: F) -> T {
        self.runtime.enter(|| RUNTIME.set(self, f))
    }

    pub fn block_on<F: Future>(&self, future: F) -> F::Output {
        self.enter(|| {
            let mut result = None;
            unsafe {
                self.runtime
                    .spawn_unchecked(async { result = Some(future.await) })
            }
            .detach();
            loop {
                self.runtime.poll_with(Some(Duration::ZERO));

                let remaining_tasks = self.runtime.run();
                if let Some(result) = result.take() {
                    break result;
                }

                let timeout = if remaining_tasks {
                    Some(Duration::ZERO)
                } else {
                    self.runtime.current_timeout()
                };
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
                let res = unsafe { PeekMessageW(msg.as_mut_ptr(), null_mut(), 0, 0, PM_REMOVE) };
                if res != 0 {
                    let msg = unsafe { msg.assume_init() };
                    unsafe {
                        TranslateMessage(&msg);
                        DispatchMessageW(&msg);
                    }
                }
            }
        })
    }

    fn set_current_msg(&self, handle: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> bool {
        let pos = unsafe { GetMessagePos() };
        let x = pos as u16;
        let y = (pos >> 16) as u16;
        let message = MSG {
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
        let detail = match msg {
            WM_COMMAND => {
                let message = (wparam as u32 >> 16) & 0xFFFF;
                let id = wparam as u32 & 0xFFFF;
                let handle = lparam as HWND;
                Some(WindowMessageDetail::Command {
                    message,
                    id,
                    handle,
                })
            }
            WM_DRAWITEM => Some(WindowMessageDetail::DrawItem(unsafe {
                *(lparam as *const DRAWITEMSTRUCT)
            })),
            _ => None,
        };
        let full_msg = WindowMessage { message, detail };
        let completes = self.registry.borrow_mut().remove(&(handle, msg));
        if let Some(completes) = completes {
            let dealt = !completes.is_empty();
            let mut futures = self.futures.borrow_mut();
            for id in completes {
                let state = futures.get_mut(id).expect("cannot find registered future");
                let state = std::mem::replace(&mut state.state, FutureState::Completed(full_msg));
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

    fn replace_waker(&self, id: usize, waker: &Waker) -> Option<WindowMessage> {
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

/// # Safety
/// The caller should ensure the handle valid.
pub unsafe fn wait(handle: HWND, msg: u32) -> impl Future<Output = WindowMessage> {
    RUNTIME.with(|runtime| runtime.register_message(handle, msg))
}

pub(crate) unsafe extern "system" fn window_proc(
    handle: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    trace!("window_proc: {:p}, {}, {}, {}", handle, msg, wparam, lparam);
    let res = RUNTIME.with(|runtime| {
        let res = runtime.set_current_msg(handle, msg, wparam, lparam);
        runtime.runtime.run();
        res
    });
    if res {
        0
    } else {
        // These messages need special return values.
        match msg {
            WM_SETTINGCHANGE => {
                window_use_dark_mode(handle);
                children_refresh_dark_mode(handle, 0);
                InvalidateRect(handle, null(), 1);
            }
            WM_CTLCOLORSTATIC => {
                return control_color_static(lparam as HWND, wparam as HDC);
            }
            WM_CTLCOLORBTN => {
                if is_dark_mode_allowed_for_app() {
                    return GetStockObject(BLACK_BRUSH) as _;
                }
            }
            WM_CTLCOLOREDIT | WM_CTLCOLORLISTBOX => {
                if is_dark_mode_allowed_for_app() {
                    let hdc = wparam as HDC;
                    let hedit = lparam as HWND;
                    SetTextColor(hdc, WHITE);
                    SetBkColor(hdc, BLACK);
                    let mut p = MaybeUninit::uninit();
                    GetCursorPos(p.as_mut_ptr());
                    let mut p = p.assume_init();
                    ScreenToClient(hedit, &mut p);
                    let is_hover = std::ptr::eq(hedit, ChildWindowFromPoint(handle, p));
                    return if is_hover {
                        GetStockObject(BLACK_BRUSH)
                    } else {
                        EDIT_NORMAL_BACK.0
                    } as _;
                }
            }
            WM_CREATE => {
                refresh_font(handle);
            }
            WM_DPICHANGED => {
                unsafe {
                    let new_rect = lparam as *const RECT;
                    if let Some(new_rect) = new_rect.as_ref() {
                        SetWindowPos(
                            handle,
                            null_mut(),
                            new_rect.left,
                            new_rect.top,
                            new_rect.right - new_rect.left,
                            new_rect.bottom - new_rect.top,
                            SWP_NOZORDER | SWP_NOACTIVATE,
                        );
                    }
                }
                refresh_font(handle);
            }
            _ => {}
        }
        DefWindowProcW(handle, msg, wparam, lparam)
    }
}

pub(crate) unsafe fn refresh_font(handle: HWND) {
    let font = default_font(get_dpi_for_window(handle));

    unsafe extern "system" fn enum_callback(hwnd: HWND, lparam: LPARAM) -> BOOL {
        SendMessageW(hwnd, WM_SETFONT, lparam as _, 1);
        EnumChildWindows(hwnd, Some(enum_callback), lparam);
        1
    }

    enum_callback(handle, font as _);
}

const WHITE: COLORREF = 0x00FFFFFF;
const BLACK: COLORREF = 0x00000000;

static EDIT_NORMAL_BACK: LazyLock<WinBrush> =
    LazyLock::new(|| WinBrush(unsafe { CreateSolidBrush(0x00212121) }));

struct MsgFuture {
    id: usize,
}

impl Future for MsgFuture {
    type Output = WindowMessage;

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
