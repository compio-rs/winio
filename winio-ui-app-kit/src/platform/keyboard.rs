use std::cell::Cell;

use objc2_app_kit::{self as appkit, NSEvent, NSEventModifierFlags};
use winio_callback::Callback;
use winio_primitive::KeyCode;

use super::key_codes as vk;
use crate::{GlobalRuntime, KeyCharCallback, character_key, from_nsstring};

const ESCAPE_CHARACTER: u32 = '\u{1b}' as u32;

// These virtual key codes identify non-text keys independently of the layout.
pub(crate) fn key_code(event: &NSEvent) -> KeyCode {
    let character = event
        .charactersIgnoringModifiers()
        .map(|s| event_character_key(&from_nsstring(&s)))
        .unwrap_or(KeyCode::Unidentified);
    // AppKit may translate combinations such as Fn+Left to a named key (Home).
    if !matches!(character, KeyCode::Char(_) | KeyCode::Unidentified) {
        return character;
    }
    match event.keyCode() {
        vk::kVK_Return | vk::kVK_ANSI_KeypadEnter => KeyCode::Enter,
        vk::kVK_Tab => KeyCode::Tab,
        vk::kVK_Delete => KeyCode::Backspace,
        vk::kVK_Escape => KeyCode::Esc,
        vk::kVK_RightCommand | vk::kVK_Command => KeyCode::Super,
        vk::kVK_Shift | vk::kVK_RightShift => KeyCode::Shift,
        vk::kVK_CapsLock => KeyCode::CapsLock,
        vk::kVK_Option | vk::kVK_RightOption => KeyCode::Alt,
        vk::kVK_Control | vk::kVK_RightControl => KeyCode::Control,
        vk::kVK_ANSI_KeypadClear => KeyCode::Clear,
        vk::CONTEXT_MENU => KeyCode::Menu,
        vk::kVK_Help => KeyCode::Insert,
        vk::kVK_Home => KeyCode::Home,
        vk::kVK_PageUp => KeyCode::PageUp,
        vk::kVK_ForwardDelete => KeyCode::Delete,
        vk::kVK_End => KeyCode::End,
        vk::kVK_PageDown => KeyCode::PageDown,
        vk::kVK_LeftArrow => KeyCode::Left,
        vk::kVK_RightArrow => KeyCode::Right,
        vk::kVK_DownArrow => KeyCode::Down,
        vk::kVK_UpArrow => KeyCode::Up,
        vk::kVK_F1 => KeyCode::F(1),
        vk::kVK_F2 => KeyCode::F(2),
        vk::kVK_F3 => KeyCode::F(3),
        vk::kVK_F4 => KeyCode::F(4),
        vk::kVK_F5 => KeyCode::F(5),
        vk::kVK_F6 => KeyCode::F(6),
        vk::kVK_F7 => KeyCode::F(7),
        vk::kVK_F8 => KeyCode::F(8),
        vk::kVK_F9 => KeyCode::F(9),
        vk::kVK_F10 => KeyCode::F(10),
        vk::kVK_F11 => KeyCode::F(11),
        vk::kVK_F12 => KeyCode::F(12),
        vk::kVK_F13 => KeyCode::F(13),
        vk::kVK_F14 => KeyCode::F(14),
        vk::kVK_F15 => KeyCode::F(15),
        vk::kVK_F16 => KeyCode::F(16),
        vk::kVK_F17 => KeyCode::F(17),
        vk::kVK_F18 => KeyCode::F(18),
        vk::kVK_F19 => KeyCode::F(19),
        vk::kVK_F20 => KeyCode::F(20),
        _ => character,
    }
}

fn event_character_key(text: &str) -> KeyCode {
    let mut chars = text.chars();
    let Some(c) = chars.next() else {
        return KeyCode::Unidentified;
    };
    if chars.next().is_some() {
        return KeyCode::Unidentified;
    }
    match c as u32 {
        appkit::NSBackspaceCharacter | appkit::NSDeleteCharacter => KeyCode::Backspace,
        appkit::NSTabCharacter | appkit::NSBackTabCharacter => KeyCode::Tab,
        appkit::NSEnterCharacter | appkit::NSCarriageReturnCharacter => KeyCode::Enter,
        ESCAPE_CHARACTER => KeyCode::Esc,
        appkit::NSUpArrowFunctionKey => KeyCode::Up,
        appkit::NSDownArrowFunctionKey => KeyCode::Down,
        appkit::NSLeftArrowFunctionKey => KeyCode::Left,
        appkit::NSRightArrowFunctionKey => KeyCode::Right,
        appkit::NSF1FunctionKey..=appkit::NSF35FunctionKey => {
            KeyCode::F((c as u32 - appkit::NSF1FunctionKey + 1) as u8)
        }
        appkit::NSInsertFunctionKey => KeyCode::Insert,
        appkit::NSDeleteFunctionKey => KeyCode::Delete,
        appkit::NSHomeFunctionKey => KeyCode::Home,
        appkit::NSBeginFunctionKey => KeyCode::Clear,
        appkit::NSEndFunctionKey => KeyCode::End,
        appkit::NSPageUpFunctionKey => KeyCode::PageUp,
        appkit::NSPageDownFunctionKey => KeyCode::PageDown,
        appkit::NSPrintScreenFunctionKey | appkit::NSSysReqFunctionKey => KeyCode::PrintScreen,
        appkit::NSScrollLockFunctionKey => KeyCode::ScrollLock,
        appkit::NSPauseFunctionKey | appkit::NSBreakFunctionKey => KeyCode::Pause,
        appkit::NSMenuFunctionKey => KeyCode::Menu,
        _ => character_key(text),
    }
}

const MODIFIERS: [(NSEventModifierFlags, KeyCode); 4] = [
    (NSEventModifierFlags::Shift, KeyCode::Shift),
    (NSEventModifierFlags::Control, KeyCode::Control),
    (NSEventModifierFlags::Option, KeyCode::Alt),
    (NSEventModifierFlags::Command, KeyCode::Super),
];

#[derive(Debug, Default)]
pub(crate) struct Keyboard {
    key_down: Callback<KeyCode>,
    key_up: Callback<KeyCode>,
    key_char: KeyCharCallback,
    modifiers: Cell<usize>,
}

impl Keyboard {
    pub fn key_down(&self, event: &NSEvent) {
        self.key_down.signal::<GlobalRuntime>(key_code(event));
    }

    pub fn key_up(&self, event: &NSEvent) {
        self.key_up.signal::<GlobalRuntime>(key_code(event));
    }

    pub fn key_char(&self, event: &NSEvent) {
        if event
            .modifierFlags()
            .contains(NSEventModifierFlags::Command)
        {
            return;
        }
        if let Some(text) = event.characters() {
            self.key_char.signal(&text);
        }
    }

    pub fn sync_modifiers(&self, flags: NSEventModifierFlags) {
        self.modifiers.set(flags.bits());
    }

    pub fn flags_changed(&self, flags: NSEventModifierFlags) {
        let previous = NSEventModifierFlags::from_bits_retain(self.modifiers.replace(flags.bits()));
        // Aggregate left/right modifiers so releasing one Shift key while the
        // other is held does not report that Shift is no longer pressed.
        for (flag, code) in MODIFIERS {
            if previous.contains(flag) != flags.contains(flag) {
                if flags.contains(flag) {
                    self.key_down.signal::<GlobalRuntime>(code);
                } else {
                    self.key_up.signal::<GlobalRuntime>(code);
                }
            }
        }
        // AppKit reports Caps Lock toggles rather than physical releases.
        if previous.contains(NSEventModifierFlags::CapsLock)
            != flags.contains(NSEventModifierFlags::CapsLock)
        {
            self.key_down.signal::<GlobalRuntime>(KeyCode::CapsLock);
            self.key_up.signal::<GlobalRuntime>(KeyCode::CapsLock);
        }
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
