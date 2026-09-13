use std::rc::Rc;

use windows_sys::Win32::{
    Foundation::{HWND, LPARAM, LRESULT, WPARAM},
    UI::{
        Shell::{DefSubclassProc, RemoveWindowSubclass, SetWindowSubclass},
        WindowsAndMessaging::{
            DLGC_WANTALLKEYS, DLGC_WANTARROWS, DLGC_WANTCHARS, DLGC_WANTTAB, WM_CHAR,
            WM_GETDLGCODE, WM_KEYDOWN, WM_KEYUP, WM_NCDESTROY, WM_SYSCHAR, WM_SYSKEYDOWN,
            WM_SYSKEYUP,
        },
    },
};
use winio_callback::Callback;
use winio_pollable::GlobalRuntime;
use winio_primitive::KeyCode;
use winio_ui_windows_common::{KeyCharCallback, key_code, syscall};

use crate::Result;

#[derive(Debug)]
pub(crate) struct Keyboard {
    hwnd: HWND,
    state: Rc<KeyboardState>,
}

impl Keyboard {
    pub fn new(hwnd: HWND) -> Result<Self> {
        let state = Rc::new(KeyboardState::default());
        syscall!(
            BOOL,
            SetWindowSubclass(
                hwnd,
                Some(keyboard_wnd_proc),
                0,
                Rc::as_ptr(&state) as usize
            )
        )?;
        Ok(Self { hwnd, state })
    }

    pub async fn wait_key_down(&self) -> KeyCode {
        self.state.key_down.wait().await
    }

    pub async fn wait_key_up(&self) -> KeyCode {
        self.state.key_up.wait().await
    }

    pub async fn wait_key_char(&self) -> char {
        self.state.chars.wait().await
    }
}

impl Drop for Keyboard {
    fn drop(&mut self) {
        unsafe { RemoveWindowSubclass(self.hwnd, Some(keyboard_wnd_proc), 0) };
    }
}

#[derive(Debug, Default)]
struct KeyboardState {
    key_down: Callback<KeyCode>,
    key_up: Callback<KeyCode>,
    chars: KeyCharCallback,
}

unsafe extern "system" fn keyboard_wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
    id: usize,
    data: usize,
) -> LRESULT {
    // The owner keeps this allocation alive while the subclass is installed.
    // Retain it for callbacks that may run user code and destroy the canvas.
    let state = data as *const KeyboardState;
    unsafe { Rc::increment_strong_count(state) };
    let state = unsafe { Rc::from_raw(state) };
    match msg {
        WM_GETDLGCODE => {
            (DLGC_WANTALLKEYS | DLGC_WANTARROWS | DLGC_WANTCHARS | DLGC_WANTTAB) as LRESULT
        }
        WM_KEYDOWN | WM_SYSKEYDOWN | WM_KEYUP | WM_SYSKEYUP => {
            let code = u16::try_from(wparam).map_or(KeyCode::Unidentified, key_code);
            // Preserve native system shortcuts such as Alt+F4.
            let result = if matches!(msg, WM_SYSKEYDOWN | WM_SYSKEYUP) {
                unsafe { DefSubclassProc(hwnd, msg, wparam, lparam) }
            } else {
                0
            };
            if matches!(msg, WM_KEYDOWN | WM_SYSKEYDOWN) {
                state.key_down.signal::<GlobalRuntime>(code);
            } else {
                state.key_up.signal::<GlobalRuntime>(code);
            }
            result
        }
        WM_CHAR | WM_SYSCHAR => {
            let result = if msg == WM_SYSCHAR {
                unsafe { DefSubclassProc(hwnd, msg, wparam, lparam) }
            } else {
                0
            };
            let repeat = (lparam as usize) & 0xffff;
            state.chars.signal_utf16(wparam as u16, repeat);
            result
        }
        WM_NCDESTROY => unsafe {
            RemoveWindowSubclass(hwnd, Some(keyboard_wnd_proc), id);
            DefSubclassProc(hwnd, msg, wparam, lparam)
        },
        _ => unsafe { DefSubclassProc(hwnd, msg, wparam, lparam) },
    }
}
