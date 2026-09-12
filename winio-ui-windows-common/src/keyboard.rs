use std::{cell::RefCell, collections::VecDeque};

use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
    self as key, MAPVK_VK_TO_CHAR, MapVirtualKeyW,
};
use winio_callback::Callback;
use winio_pollable::GlobalRuntime;
use winio_primitive::KeyCode;

/// Convert a Windows virtual key to a portable key code using the current
/// layout.
pub fn key_code(code: u16) -> KeyCode {
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
            // Mapping does not alter the keyboard's dead-key state. The high
            // bit marks a dead key rather than part of the Unicode value.
            const DEAD_KEY_FLAG: u32 = 1 << 31;
            let c = unsafe { MapVirtualKeyW(code.into(), MAPVK_VK_TO_CHAR) } & !DEAD_KEY_FLAG;
            char::from_u32(c)
                .filter(|c| !c.is_control())
                .map_or(KeyCode::Unidentified, KeyCode::Char)
        }
    }
}

/// Buffers native character messages, including partial UTF-16 surrogate pairs.
#[derive(Debug, Default)]
pub struct KeyCharCallback {
    chars: RefCell<CharBuffer>,
    ready: Callback,
}

impl KeyCharCallback {
    /// Enqueue a UTF-16 code unit. A zero repeat count is treated as one.
    pub fn signal_utf16(&self, unit: u16, repeat: usize) {
        self.chars.borrow_mut().push_utf16(unit, repeat.max(1));
        self.notify();
    }

    /// Enqueue a Unicode character. A zero repeat count is treated as one.
    pub fn signal_char(&self, c: char, repeat: usize) {
        self.chars.borrow_mut().push_char(c, repeat.max(1));
        self.notify();
    }

    fn notify(&self) {
        if !self.chars.borrow().pending.is_empty() {
            self.ready.signal::<GlobalRuntime>(());
        }
    }

    /// Wait for a character, preserving partial pairs and queued text on
    /// cancellation.
    pub async fn wait(&self) -> char {
        loop {
            if let Some(c) = self.chars.borrow_mut().pending.pop_front() {
                return c;
            }
            self.ready.wait().await;
        }
    }
}

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
