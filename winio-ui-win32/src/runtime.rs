use std::{
    cell::RefCell,
    collections::HashMap,
    future::Future,
    mem::MaybeUninit,
    pin::Pin,
    ptr::{null, null_mut},
    task::{Context, Poll, Waker},
};

use compio_log::*;
use slab::Slab;
use windows::Win32::Graphics::Direct2D::ID2D1Factory2;
#[cfg(target_pointer_width = "64")]
use windows_sys::Win32::UI::WindowsAndMessaging::SetClassLongPtrW;
#[cfg(not(target_pointer_width = "64"))]
use windows_sys::Win32::UI::WindowsAndMessaging::SetClassLongW as SetClassLongPtrW;
use windows_sys::{
    Win32::{
        Foundation::{HWND, LPARAM, LRESULT, RECT, WPARAM},
        Graphics::{
            Dwm::DwmExtendFrameIntoClientArea,
            Gdi::{BLACK_BRUSH, GetStockObject, HDC, InvalidateRect, WHITE_BRUSH},
        },
        UI::{
            Controls::{MARGINS, NMHDR},
            WindowsAndMessaging::{
                DefWindowProcW, DispatchMessageW, EnumChildWindows, GA_ROOT, GCLP_HBRBACKGROUND,
                GetAncestor, IsDialogMessageW, PostQuitMessage, SWP_NOACTIVATE, SWP_NOZORDER,
                SendMessageW, SetWindowPos, TranslateMessage, WM_COMMAND, WM_CREATE,
                WM_CTLCOLORBTN, WM_CTLCOLOREDIT, WM_CTLCOLORLISTBOX, WM_CTLCOLORSTATIC,
                WM_DPICHANGED, WM_NOTIFY, WM_SETFONT, WM_SETTINGCHANGE,
            },
        },
    },
    core::BOOL,
};
use winio_ui_windows_common::{
    Backdrop, children_refresh_dark_mode, control_color_edit, control_color_static, init_dark,
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
    runtime: winio_ui_windows_common::Runtime,
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

        let runtime = winio_ui_windows_common::Runtime::new().unwrap();

        Self {
            runtime,
            registry: RefCell::new(HashMap::new()),
        }
    }

    pub(crate) fn d2d1(&self) -> &ID2D1Factory2 {
        self.runtime.d2d1()
    }

    fn enter<T, F: FnOnce() -> T>(&self, f: F) -> T {
        self.runtime.enter(|| RUNTIME.set(self, f))
    }

    pub fn block_on<F: Future>(&self, future: F) -> F::Output {
        self.enter(|| {
            let mut result = None;
            unsafe {
                self.runtime.spawn_unchecked(async {
                    result = Some(future.await);
                    PostQuitMessage(0);
                })
            }
            .detach();

            loop {
                let mut msg = MaybeUninit::uninit();
                let res = unsafe { self.runtime.get_message(msg.as_mut_ptr(), null_mut(), 0, 0) };
                if res > 0 {
                    let msg = unsafe { msg.assume_init() };
                    unsafe {
                        let root = GetAncestor(msg.hwnd, GA_ROOT);
                        let handled = !root.is_null() && (IsDialogMessageW(root, &msg) != 0);
                        if !handled {
                            TranslateMessage(&msg);
                            DispatchMessageW(&msg);
                        }
                    }
                } else if res == 0 {
                    debug!("received WM_QUIT");
                    break result.take().expect("received WM_QUIT but no result");
                } else {
                    panic!("{:?}", std::io::Error::last_os_error());
                }
            }
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

pub(crate) unsafe extern "system" fn window_proc(
    handle: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    trace!("window_proc: {:p}, {}, {}, {}", handle, msg, wparam, lparam);
    // These messages need special handling.
    match msg {
        WM_CREATE => {
            window_use_dark_mode(handle);
            refresh_background(handle);
        }
        WM_SETTINGCHANGE => {
            window_use_dark_mode(handle);
            children_refresh_dark_mode(handle, 0);
            refresh_background(handle);
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

pub(crate) unsafe fn set_backdrop(handle: HWND, backdrop: Backdrop) {
    let old_backdrop = unsafe { get_backdrop(handle) };
    if old_backdrop != backdrop {
        set_backdrop_impl(handle, backdrop);
        refresh_background(handle);
    }
}

pub(crate) use winio_ui_windows_common::get_backdrop;

unsafe fn set_backdrop_impl(handle: HWND, backdrop: Backdrop) {
    let res = winio_ui_windows_common::set_backdrop(handle, backdrop);
    if res {
        let margins = MARGINS {
            cxLeftWidth: -1,
            cxRightWidth: -1,
            cyTopHeight: -1,
            cyBottomHeight: -1,
        };
        DwmExtendFrameIntoClientArea(handle, &margins);
    } else {
        let margins = MARGINS {
            cxLeftWidth: 0,
            cxRightWidth: 0,
            cyTopHeight: 0,
            cyBottomHeight: 0,
        };
        DwmExtendFrameIntoClientArea(handle, &margins);
    }
}

unsafe fn refresh_background(handle: HWND) {
    let backdrop = get_backdrop(GetAncestor(handle, GA_ROOT));
    let black = !matches!(backdrop, Backdrop::None) || is_dark_mode_allowed_for_app();
    let brush = if black {
        GetStockObject(BLACK_BRUSH)
    } else {
        GetStockObject(WHITE_BRUSH)
    };
    SetClassLongPtrW(handle, GCLP_HBRBACKGROUND, brush as _);
}
