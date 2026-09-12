use std::{
    cell::{Cell, RefCell},
    collections::VecDeque,
};

use objc2_app_kit::{self as appkit, NSEvent, NSEventModifierFlags};
use winio_callback::Callback;
use winio_primitive::KeyCode;

use crate::{GlobalRuntime, from_nsstring};

// These virtual key codes identify non-text keys independently of the layout.
pub(crate) fn key_code(event: &NSEvent) -> KeyCode {
    let character = event
        .charactersIgnoringModifiers()
        .map(|s| character_key(&from_nsstring(&s)))
        .unwrap_or(KeyCode::Unidentified);
    // AppKit may translate combinations such as Fn+Left to a named key (Home).
    if !matches!(character, KeyCode::Char(_) | KeyCode::Unidentified) {
        return character;
    }
    match event.keyCode() {
        0x24 | 0x4c => KeyCode::Enter,
        0x30 => KeyCode::Tab,
        0x33 => KeyCode::Backspace,
        0x35 => KeyCode::Esc,
        0x36 | 0x37 => KeyCode::Super,
        0x38 | 0x3c => KeyCode::Shift,
        0x39 => KeyCode::CapsLock,
        0x3a | 0x3d => KeyCode::Alt,
        0x3b | 0x3e => KeyCode::Control,
        0x47 => KeyCode::Clear,
        0x6e => KeyCode::Menu,
        0x72 => KeyCode::Insert,
        0x73 => KeyCode::Home,
        0x74 => KeyCode::PageUp,
        0x75 => KeyCode::Delete,
        0x77 => KeyCode::End,
        0x79 => KeyCode::PageDown,
        0x7b => KeyCode::Left,
        0x7c => KeyCode::Right,
        0x7d => KeyCode::Down,
        0x7e => KeyCode::Up,
        0x7a => KeyCode::F(1),
        0x78 => KeyCode::F(2),
        0x63 => KeyCode::F(3),
        0x76 => KeyCode::F(4),
        0x60 => KeyCode::F(5),
        0x61 => KeyCode::F(6),
        0x62 => KeyCode::F(7),
        0x64 => KeyCode::F(8),
        0x65 => KeyCode::F(9),
        0x6d => KeyCode::F(10),
        0x67 => KeyCode::F(11),
        0x6f => KeyCode::F(12),
        0x69 => KeyCode::F(13),
        0x6b => KeyCode::F(14),
        0x71 => KeyCode::F(15),
        0x6a => KeyCode::F(16),
        0x40 => KeyCode::F(17),
        0x4f => KeyCode::F(18),
        0x50 => KeyCode::F(19),
        0x5a => KeyCode::F(20),
        _ => character,
    }
}

fn character_key(text: &str) -> KeyCode {
    let mut chars = text.chars();
    let Some(c) = chars.next() else {
        return KeyCode::Unidentified;
    };
    if chars.next().is_some() {
        return KeyCode::Unidentified;
    }
    match c as u32 {
        0x08 | 0x7f => KeyCode::Backspace,
        0x09 | 0x19 => KeyCode::Tab,
        0x03 | 0x0d => KeyCode::Enter,
        0x1b => KeyCode::Esc,
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
        _ => {
            let mut upper = c.to_uppercase();
            let c = upper.next().unwrap();
            // Unmapped AppKit function-key symbols are not character keys.
            if c.is_control() || ('\u{f700}'..='\u{f8ff}').contains(&c) || upper.next().is_some() {
                return KeyCode::Unidentified;
            }
            KeyCode::Char(c)
        }
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
    pending: RefCell<VecDeque<char>>,
    ready: Callback,
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
            let text = from_nsstring(&text);
            // AppKit encodes function keys in this private-use range.
            let mut chars = text
                .chars()
                .filter(|c| !('\u{f700}'..='\u{f8ff}').contains(c))
                .peekable();
            if chars.peek().is_some() {
                self.pending.borrow_mut().extend(chars);
                self.ready.signal::<GlobalRuntime>(());
            }
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
        loop {
            if let Some(c) = self.pending.borrow_mut().pop_front() {
                return c;
            }
            self.ready.wait().await;
        }
    }
}
