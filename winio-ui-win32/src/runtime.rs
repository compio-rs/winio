use std::{
    cell::{Cell, OnceCell, RefCell},
    collections::HashMap,
    future::Future,
    mem::MaybeUninit,
    pin::Pin,
    ptr::{null, null_mut},
    task::{Context, Poll, Waker},
};

use compio::driver::AsRawFd;
use compio_log::*;
use slab::Slab;
use windows::Win32::Graphics::Direct2D::{
    D2D1_FACTORY_TYPE_SINGLE_THREADED, D2D1CreateFactory, ID2D1Factory,
};
use windows_sys::{
    Win32::{
        Foundation::{HANDLE, HWND, LPARAM, LRESULT, RECT, WAIT_FAILED, WPARAM},
        Graphics::Gdi::{BLACK_BRUSH, GetStockObject, HDC, InvalidateRect},
        System::Threading::INFINITE,
        UI::{
            Controls::NMHDR,
            Input::KeyboardAndMouse::{GetFocus, SetFocus},
            WindowsAndMessaging::{
                DefWindowProcW, DispatchMessageW, EnumChildWindows, GA_ROOT, GetAncestor,
                IsDialogMessageW, MWMO_ALERTABLE, MWMO_INPUTAVAILABLE, MsgWaitForMultipleObjectsEx,
                PM_REMOVE, PeekMessageW, QS_ALLINPUT, SWP_NOACTIVATE, SWP_NOZORDER, SendMessageW,
                SetWindowPos, TranslateMessage, WA_INACTIVE, WM_ACTIVATE, WM_COMMAND, WM_CREATE,
                WM_CTLCOLORBTN, WM_CTLCOLOREDIT, WM_CTLCOLORLISTBOX, WM_CTLCOLORSTATIC,
                WM_DPICHANGED, WM_NOTIFY, WM_SETFOCUS, WM_SETFONT, WM_SETTINGCHANGE, WM_USER,
            },
        },
    },
    core::BOOL,
};
use winio_ui_windows_common::{
    children_refresh_dark_mode, control_color_edit, control_color_static, init_dark,
    is_dark_mode_allowed_for_app, window_use_dark_mode,
};

use super::RUNTIME;
use crate::ui::{dpi::get_dpi_for_window, font::default_font};

#[derive(Clone, Copy)]
pub(crate) enum WindowMessage {
    General {
        // handle: HWND,
        // message: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    },
    Command(WindowMessageCommand),
    Notify(WindowMessageNotify),
}

impl WindowMessage {
    pub fn wparam(&self) -> WPARAM {
        match self {
            Self::General { wparam, .. } => *wparam,
            _ => unreachable!(),
        }
    }

    pub fn lparam(&self) -> LPARAM {
        match self {
            Self::General { lparam, .. } => *lparam,
            _ => unreachable!(),
        }
    }

    pub fn command(self) -> WindowMessageCommand {
        match self {
            Self::Command(c) => c,
            _ => unreachable!(),
        }
    }

    pub fn notify(self) -> WindowMessageNotify {
        match self {
            Self::Notify(n) => n,
            _ => unreachable!(),
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) struct WindowMessageCommand {
    pub message: u32,
    // pub id: usize,
    pub handle: HWND,
}

#[derive(Clone, Copy)]
pub(crate) struct WindowMessageNotify {
    pub hwnd_from: HWND,
    // pub id_from: usize,
    pub code: u32,
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

pub struct Runtime {
    runtime: winio_pollable::Runtime,
    d2d1: OnceCell<ID2D1Factory>,
    registry: RefCell<HashMap<(HWND, u32), Slab<FutureState>>>,
}

impl Default for Runtime {
    fn default() -> Self {
        Self::new()
    }
}

impl Runtime {
    pub fn new() -> Self {
        init_dark();

        let runtime = winio_pollable::Runtime::new().unwrap();

        Self {
            runtime,
            d2d1: OnceCell::new(),
            registry: RefCell::new(HashMap::new()),
        }
    }

    pub(crate) fn d2d1(&self) -> &ID2D1Factory {
        self.d2d1.get_or_init(|| unsafe {
            D2D1CreateFactory(D2D1_FACTORY_TYPE_SINGLE_THREADED, None).unwrap()
        })
    }

    fn enter<T, F: FnOnce() -> T>(&self, f: F) -> T {
        self.runtime.enter(|| RUNTIME.set(self, f))
    }

    pub fn block_on<F: Future>(&self, future: F) -> F::Output {
        self.enter(|| {
            self.runtime.block_on(future, |timeout| {
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

                loop {
                    let mut msg = MaybeUninit::uninit();
                    let res =
                        unsafe { PeekMessageW(msg.as_mut_ptr(), null_mut(), 0, 0, PM_REMOVE) };
                    if res != 0 {
                        let msg = unsafe { msg.assume_init() };
                        unsafe {
                            let root = GetAncestor(msg.hwnd, GA_ROOT);
                            let handled = !root.is_null() && (IsDialogMessageW(root, &msg) != 0);
                            if !handled {
                                TranslateMessage(&msg);
                                DispatchMessageW(&msg);
                            }
                        }
                    } else {
                        break;
                    }
                }
            })
        })
    }

    fn set_current_msg(&self, handle: HWND, message: u32, wparam: WPARAM, lparam: LPARAM) -> bool {
        let full_msg = match message {
            WM_COMMAND => WindowMessage::Command({
                let message = (wparam as u32 >> 16) & 0xFFFF;
                // let id = wparam as u32 & 0xFFFF;
                let handle = lparam as HWND;
                WindowMessageCommand {
                    message,
                    // id: id as _,
                    handle,
                }
            }),
            WM_NOTIFY => WindowMessage::Notify(unsafe {
                let header = &*(lparam as *const NMHDR);
                WindowMessageNotify {
                    hwnd_from: header.hwndFrom,
                    // id_from: header.idFrom,
                    code: header.code,
                }
            }),
            WM_USER_CALLBACK => {
                let callback = unsafe { UserCallback::from_raw(lparam) };
                callback.call();
                return true;
            }
            _ => WindowMessage::General { wparam, lparam },
        };
        let mut registry = self.registry.borrow_mut();
        let completes = registry.get_mut(&(handle, message));
        if let Some(completes) = completes {
            let dealt = !completes.is_empty();
            for (_, state) in completes {
                let state = std::mem::replace(state, FutureState::Completed(full_msg));
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
            .registry
            .borrow_mut()
            .entry((handle, msg))
            .or_default()
            .insert(FutureState::Active(None));
        debug!("register: {}", id);
        MsgFuture { id, handle, msg }
    }

    fn replace_waker(
        &self,
        id: usize,
        handle: HWND,
        msg: u32,
        waker: &Waker,
    ) -> Option<WindowMessage> {
        if let Some(futures) = self.registry.borrow_mut().get_mut(&(handle, msg)) {
            if let Some(state) = futures.get_mut(id) {
                if let FutureState::Completed(msg) = *state {
                    return Some(msg);
                } else {
                    *state = FutureState::Active(Some(waker.clone()));
                }
            }
        }
        None
    }

    fn deregister(&self, id: usize, handle: HWND, msg: u32) {
        if let Some(futures) = self.registry.borrow_mut().get_mut(&(handle, msg)) {
            futures.remove(id);
        }
    }
}

/// # Safety
/// The caller should ensure the handle valid.
pub(crate) unsafe fn wait(handle: HWND, msg: u32) -> impl Future<Output = WindowMessage> {
    RUNTIME.with(|runtime| runtime.register_message(handle, msg))
}

thread_local! {
    static FOCUS: Cell<HWND> = const { Cell::new(null_mut()) };
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
            WM_ACTIVATE => {
                let state = (wparam & 0xFFFF) as u32;
                if state == WA_INACTIVE {
                    FOCUS.set(GetFocus());
                }
                return 0;
            }
            WM_SETFOCUS => {
                let focus = FOCUS.get();
                if !focus.is_null() {
                    SetFocus(focus);
                }
                return 0;
            }
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
                if let Some(res) = control_color_edit(handle, lparam as HWND, wparam as HDC) {
                    return res;
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

struct MsgFuture {
    id: usize,
    handle: HWND,
    msg: u32,
}

impl Future for MsgFuture {
    type Output = WindowMessage;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        instrument!(Level::DEBUG, "MsgFuture", ?self.id);
        if let Some(msg) = RUNTIME
            .with(|runtime| runtime.replace_waker(self.id, self.handle, self.msg, cx.waker()))
        {
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
        RUNTIME.with(|runtime| runtime.deregister(self.id, self.handle, self.msg));
    }
}

/// A message that can be posted to a window to execute a callback.
/// * wParam: 0
/// * lParam: *mut Box<UserCallback>
///
/// The window procedure must call [`UserCallback::from_raw`] to take ownership
/// of the callback and then call it.
pub const WM_USER_CALLBACK: u32 = WM_USER + 1;

pub struct UserCallback {
    callback: Box<dyn FnOnce() + Send>,
}

impl UserCallback {
    pub fn new(f: impl FnOnce() + Send + 'static) -> Self {
        Self {
            callback: Box::new(f),
        }
    }

    pub fn into_raw(self) -> LPARAM {
        Box::into_raw(Box::new(self)) as _
    }

    /// # Safety
    /// The caller must ensure that lparam is from [`WM_USER_CALLBACK`].
    pub unsafe fn from_raw(lparam: LPARAM) -> Self {
        *Box::from_raw(lparam as *mut Self)
    }

    pub fn call(self) {
        (self.callback)()
    }
}
