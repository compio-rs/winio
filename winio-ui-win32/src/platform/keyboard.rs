use std::{cell::RefCell, collections::VecDeque, rc::Rc};

use windows_sys::Win32::{
    Foundation::{HWND, LPARAM, LRESULT, WPARAM},
    UI::{
        Input::KeyboardAndMouse::{self as key, MAPVK_VK_TO_CHAR, MapVirtualKeyW},
        Shell::{DefSubclassProc, RemoveWindowSubclass, SetWindowSubclass},
        WindowsAndMessaging::{
            DLGC_WANTALLKEYS, DLGC_WANTARROWS, DLGC_WANTCHARS, DLGC_WANTTAB, UNICODE_NOCHAR,
            WM_CHAR, WM_GETDLGCODE, WM_KEYDOWN, WM_KEYUP, WM_NCDESTROY, WM_SYSCHAR, WM_SYSKEYDOWN,
            WM_SYSKEYUP, WM_UNICHAR,
        },
    },
};
use winio_callback::Callback;
use winio_pollable::GlobalRuntime;
use winio_primitive::KeyCode;
use winio_ui_windows_common::syscall;

use crate::Result;

pub(crate) fn key_code(code: u16) -> KeyCode {
    match code {
        key::VK_BACK => KeyCode::Backspace,
        key::VK_RETURN => KeyCode::Enter,
        key::VK_LEFT => KeyCode::Left,
        key::VK_RIGHT => KeyCode::Right,
        key::VK_UP => KeyCode::Up,
        key::VK_DOWN => KeyCode::Down,
        key::VK_HOME => KeyCode::Home,
        key::VK_END => KeyCode::End,
        key::VK_PRIOR => KeyCode::PageUp,
        key::VK_NEXT => KeyCode::PageDown,
        key::VK_TAB => KeyCode::Tab,
        key::VK_DELETE => KeyCode::Delete,
        key::VK_INSERT => KeyCode::Insert,
        key::VK_ESCAPE => KeyCode::Esc,
        key::VK_CAPITAL => KeyCode::CapsLock,
        key::VK_SCROLL => KeyCode::ScrollLock,
        key::VK_NUMLOCK => KeyCode::NumLock,
        key::VK_SNAPSHOT => KeyCode::PrintScreen,
        key::VK_PAUSE | key::VK_CANCEL => KeyCode::Pause,
        key::VK_APPS => KeyCode::Menu,
        key::VK_CLEAR => KeyCode::Clear,
        key::VK_SHIFT | key::VK_LSHIFT | key::VK_RSHIFT => KeyCode::Shift,
        key::VK_CONTROL | key::VK_LCONTROL | key::VK_RCONTROL => KeyCode::Control,
        key::VK_MENU | key::VK_LMENU | key::VK_RMENU => KeyCode::Alt,
        key::VK_LWIN | key::VK_RWIN => KeyCode::Super,
        key::VK_F1..=key::VK_F24 => KeyCode::F((code - key::VK_F1 + 1) as u8),
        key::VK_NUMPAD0..=key::VK_NUMPAD9 => {
            KeyCode::Char(char::from(b'0' + (code - key::VK_NUMPAD0) as u8))
        }
        key::VK_MULTIPLY => KeyCode::Char('*'),
        key::VK_ADD => KeyCode::Char('+'),
        key::VK_SUBTRACT => KeyCode::Char('-'),
        key::VK_DIVIDE => KeyCode::Char('/'),
        _ => {
            // Uses the current keyboard layout and does not alter its dead-key
            // state. The high bit marks a dead key rather than part of Unicode.
            const DEAD_KEY_FLAG: u32 = 1 << 31;
            let c = unsafe { MapVirtualKeyW(code.into(), MAPVK_VK_TO_CHAR) } & !DEAD_KEY_FLAG;
            char::from_u32(c)
                .filter(|c| !c.is_control())
                .map_or(KeyCode::Unidentified, KeyCode::Char)
        }
    }
}

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
        loop {
            if let Some(c) = self.state.chars.borrow_mut().pending.pop_front() {
                return c;
            }
            self.state.ready.wait().await;
        }
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
    chars: RefCell<CharBuffer>,
    ready: Callback,
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
        WM_UNICHAR if wparam == UNICODE_NOCHAR as WPARAM => 1,
        WM_CHAR | WM_SYSCHAR | WM_UNICHAR => {
            let result = if msg == WM_SYSCHAR {
                unsafe { DefSubclassProc(hwnd, msg, wparam, lparam) }
            } else {
                0
            };
            let repeat = ((lparam as usize) & 0xffff).max(1);
            let ready = {
                let mut chars = state.chars.borrow_mut();
                if msg == WM_UNICHAR {
                    let c = u32::try_from(wparam)
                        .ok()
                        .and_then(char::from_u32)
                        .unwrap_or(char::REPLACEMENT_CHARACTER);
                    chars.push_char(c, repeat);
                } else {
                    chars.push_utf16(wparam as u16, repeat);
                }
                !chars.pending.is_empty()
            };
            if ready {
                state.ready.signal::<GlobalRuntime>(());
            }
            result
        }
        WM_NCDESTROY => unsafe {
            RemoveWindowSubclass(hwnd, Some(keyboard_wnd_proc), id);
            DefSubclassProc(hwnd, msg, wparam, lparam)
        },
        _ => unsafe { DefSubclassProc(hwnd, msg, wparam, lparam) },
    }
}

// Persist partial surrogate pairs and queued characters across cancelled waits.
#[derive(Debug, Default)]
struct CharBuffer {
    high: Option<(u16, usize)>,
    pending: VecDeque<char>,
}

impl CharBuffer {
    fn push_utf16(&mut self, unit: u16, repeat: usize) {
        let high = self.high.take();
        if (0xdc00..=0xdfff).contains(&unit)
            && let Some((high, high_repeat)) = high
        {
            let c = char::decode_utf16([high, unit]).next().unwrap().unwrap();
            self.pending.extend(std::iter::repeat_n(
                char::REPLACEMENT_CHARACTER,
                high_repeat.saturating_sub(repeat),
            ));
            self.pending
                .extend(std::iter::repeat_n(c, repeat.min(high_repeat)));
            self.pending.extend(std::iter::repeat_n(
                char::REPLACEMENT_CHARACTER,
                repeat.saturating_sub(high_repeat),
            ));
            return;
        }
        if let Some((_, count)) = high {
            self.pending
                .extend(std::iter::repeat_n(char::REPLACEMENT_CHARACTER, count));
        }
        if (0xd800..=0xdbff).contains(&unit) {
            self.high = Some((unit, repeat));
        } else {
            let c = char::from_u32(unit.into()).unwrap_or(char::REPLACEMENT_CHARACTER);
            self.pending.extend(std::iter::repeat_n(c, repeat));
        }
    }

    fn push_char(&mut self, c: char, repeat: usize) {
        if let Some((_, count)) = self.high.take() {
            self.pending
                .extend(std::iter::repeat_n(char::REPLACEMENT_CHARACTER, count));
        }
        self.pending.extend(std::iter::repeat_n(c, repeat));
    }
}
