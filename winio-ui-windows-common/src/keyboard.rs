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
    chars: RefCell<VecDeque<u16>>,
    ready: Callback,
}

impl KeyCharCallback {
    pub fn signal_utf16(&self, unit: u16, repeat: usize) {
        self.chars
            .borrow_mut()
            .extend(std::iter::repeat_n(unit, repeat.max(1)));
        self.ready.signal::<GlobalRuntime>(());
    }

    fn pop(&self) -> Option<char> {
        let mut chars = self.chars.borrow_mut();
        let first = *chars.front()?;
        // Keep a lone high surrogate until the next code unit arrives.
        if (0xd800..=0xdbff).contains(&first) && chars.len() == 1 {
            return None;
        }
        let c = char::decode_utf16(chars.iter().copied())
            .next()?
            .unwrap_or(char::REPLACEMENT_CHARACTER);
        chars.drain(..c.len_utf16());
        Some(c)
    }

    /// Wait for a character, preserving partial pairs and queued text on
    /// cancellation.
    pub async fn wait(&self) -> char {
        loop {
            if let Some(c) = self.pop() {
                return c;
            }
            self.ready.wait().await;
        }
    }
}
