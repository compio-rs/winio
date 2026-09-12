use objc2_ui_kit::{UIKey, UIKeyModifierFlags, UIKeyboardHIDUsage as Key};
use winio_callback::Callback;
use winio_primitive::KeyCode;

use crate::{GlobalRuntime, KeyCharCallback, character_key, from_nsstring};

pub(crate) fn key_code(key: &UIKey) -> KeyCode {
    match key.keyCode() {
        Key::KeyboardDeleteOrBackspace => KeyCode::Backspace,
        Key::KeyboardReturnOrEnter | Key::KeyboardReturn | Key::KeypadEnter => KeyCode::Enter,
        Key::KeyboardLeftArrow => KeyCode::Left,
        Key::KeyboardRightArrow => KeyCode::Right,
        Key::KeyboardUpArrow => KeyCode::Up,
        Key::KeyboardDownArrow => KeyCode::Down,
        Key::KeyboardHome => KeyCode::Home,
        Key::KeyboardEnd => KeyCode::End,
        Key::KeyboardPageUp => KeyCode::PageUp,
        Key::KeyboardPageDown => KeyCode::PageDown,
        Key::KeyboardTab => KeyCode::Tab,
        Key::KeyboardDeleteForward => KeyCode::Delete,
        Key::KeyboardInsert => KeyCode::Insert,
        Key::KeyboardEscape => KeyCode::Esc,
        Key::KeyboardCapsLock | Key::KeyboardLockingCapsLock => KeyCode::CapsLock,
        Key::KeyboardScrollLock | Key::KeyboardLockingScrollLock => KeyCode::ScrollLock,
        Key::KeypadNumLock | Key::KeyboardLockingNumLock => KeyCode::NumLock,
        Key::KeyboardPrintScreen | Key::KeyboardSysReqOrAttention => KeyCode::PrintScreen,
        Key::KeyboardPause => KeyCode::Pause,
        Key::KeyboardApplication | Key::KeyboardMenu => KeyCode::Menu,
        Key::KeyboardClear => KeyCode::Clear,
        Key::KeyboardLeftShift | Key::KeyboardRightShift => KeyCode::Shift,
        Key::KeyboardLeftControl | Key::KeyboardRightControl => KeyCode::Control,
        Key::KeyboardLeftAlt | Key::KeyboardRightAlt => KeyCode::Alt,
        Key::KeyboardLeftGUI | Key::KeyboardRightGUI => KeyCode::Super,
        code if (Key::KeyboardF1..=Key::KeyboardF12).contains(&code) => {
            KeyCode::F((code.0 - Key::KeyboardF1.0 + 1) as u8)
        }
        code if (Key::KeyboardF13..=Key::KeyboardF24).contains(&code) => {
            KeyCode::F((code.0 - Key::KeyboardF13.0 + 13) as u8)
        }
        _ => {
            // HID letter positions are layout-independent; use UIKit's key
            // symbol instead so non-US layouts keep their native mapping.
            character_key(&from_nsstring(&key.charactersIgnoringModifiers()))
        }
    }
}

#[derive(Debug, Default)]
pub(crate) struct Keyboard {
    key_down: Callback<KeyCode>,
    key_up: Callback<KeyCode>,
    key_char: KeyCharCallback,
}

impl Keyboard {
    pub fn key_down(&self, key: &UIKey) {
        self.key_down.signal::<GlobalRuntime>(key_code(key));
    }

    pub fn key_up(&self, key: &UIKey) {
        self.key_up.signal::<GlobalRuntime>(key_code(key));
    }

    pub fn key_char(&self, key: &UIKey) {
        if key.modifierFlags().contains(UIKeyModifierFlags::Command) {
            return;
        }
        // Navigation, function and modifier keys may carry symbolic UIKit
        // strings rather than text to be inserted.
        if !matches!(
            key_code(key),
            KeyCode::Char(_)
                | KeyCode::Unidentified
                | KeyCode::Backspace
                | KeyCode::Enter
                | KeyCode::Tab
                | KeyCode::Delete
                | KeyCode::Esc
        ) {
            return;
        }
        self.key_char.signal(&key.characters());
    }

    pub async fn wait_key_down(&self) -> KeyCode {
        self.key_down.wait().await
    }

    pub async fn wait_key_up(&self) -> KeyCode {
        self.key_up.wait().await
    }

    pub async fn wait_key_char(&self) -> char {
        self.key_char.wait().await
    }
}
