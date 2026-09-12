use std::{cell::RefCell, collections::VecDeque};

use winio_callback::Callback;
use winio_primitive::KeyCode;

use crate::{GlobalRuntime, common::QString};

const QT_KEY_ESCAPE: i32 = 0x01000000;
const QT_KEY_TAB: i32 = 0x01000001;
const QT_KEY_BACKTAB: i32 = 0x01000002;
const QT_KEY_BACKSPACE: i32 = 0x01000003;
const QT_KEY_RETURN: i32 = 0x01000004;
const QT_KEY_ENTER: i32 = 0x01000005;
const QT_KEY_INSERT: i32 = 0x01000006;
const QT_KEY_DELETE: i32 = 0x01000007;
const QT_KEY_PAUSE: i32 = 0x01000008;
const QT_KEY_PRINT: i32 = 0x01000009;
const QT_KEY_SYSREQ: i32 = 0x0100000a;
const QT_KEY_CLEAR: i32 = 0x0100000b;
const QT_KEY_HOME: i32 = 0x01000010;
const QT_KEY_END: i32 = 0x01000011;
const QT_KEY_LEFT: i32 = 0x01000012;
const QT_KEY_UP: i32 = 0x01000013;
const QT_KEY_RIGHT: i32 = 0x01000014;
const QT_KEY_DOWN: i32 = 0x01000015;
const QT_KEY_PAGE_UP: i32 = 0x01000016;
const QT_KEY_PAGE_DOWN: i32 = 0x01000017;
const QT_KEY_SHIFT: i32 = 0x01000020;
const QT_KEY_CONTROL: i32 = 0x01000021;
const QT_KEY_META: i32 = 0x01000022;
const QT_KEY_ALT: i32 = 0x01000023;
const QT_KEY_CAPS_LOCK: i32 = 0x01000024;
const QT_KEY_NUM_LOCK: i32 = 0x01000025;
const QT_KEY_SCROLL_LOCK: i32 = 0x01000026;
const QT_KEY_F1: i32 = 0x01000030;
const QT_KEY_F35: i32 = 0x01000052;
const QT_KEY_SUPER_L: i32 = 0x01000053;
const QT_KEY_SUPER_R: i32 = 0x01000054;
const QT_KEY_MENU: i32 = 0x01000055;
const QT_KEY_HYPER_L: i32 = 0x01000056;
const QT_KEY_HYPER_R: i32 = 0x01000057;
const QT_KEY_ALTGR: i32 = 0x01001103;

pub(crate) fn key_code(key: i32) -> KeyCode {
    match key {
        QT_KEY_BACKSPACE => KeyCode::Backspace,
        QT_KEY_RETURN | QT_KEY_ENTER => KeyCode::Enter,
        QT_KEY_LEFT => KeyCode::Left,
        QT_KEY_RIGHT => KeyCode::Right,
        QT_KEY_UP => KeyCode::Up,
        QT_KEY_DOWN => KeyCode::Down,
        QT_KEY_HOME => KeyCode::Home,
        QT_KEY_END => KeyCode::End,
        QT_KEY_PAGE_UP => KeyCode::PageUp,
        QT_KEY_PAGE_DOWN => KeyCode::PageDown,
        QT_KEY_TAB | QT_KEY_BACKTAB => KeyCode::Tab,
        QT_KEY_DELETE => KeyCode::Delete,
        QT_KEY_INSERT => KeyCode::Insert,
        QT_KEY_ESCAPE => KeyCode::Esc,
        QT_KEY_CAPS_LOCK => KeyCode::CapsLock,
        QT_KEY_SCROLL_LOCK => KeyCode::ScrollLock,
        QT_KEY_NUM_LOCK => KeyCode::NumLock,
        QT_KEY_PRINT | QT_KEY_SYSREQ => KeyCode::PrintScreen,
        QT_KEY_PAUSE => KeyCode::Pause,
        QT_KEY_MENU => KeyCode::Menu,
        QT_KEY_CLEAR => KeyCode::Clear,
        QT_KEY_SHIFT => KeyCode::Shift,
        QT_KEY_CONTROL => KeyCode::Control,
        QT_KEY_ALT => KeyCode::Alt,
        QT_KEY_ALTGR => KeyCode::AltGr,
        QT_KEY_META => KeyCode::Meta,
        QT_KEY_SUPER_L | QT_KEY_SUPER_R => KeyCode::Super,
        QT_KEY_HYPER_L | QT_KEY_HYPER_R => KeyCode::Hyper,
        QT_KEY_F1..=QT_KEY_F35 => KeyCode::F((key - QT_KEY_F1 + 1) as u8),
        // Qt uses uppercase letter symbols regardless of Shift or Caps Lock.
        0x20..=0x7e | 0xa0..=0xff => KeyCode::Char(key as u8),
        _ => KeyCode::Unidentified,
    }
}

// A native text event can contain several Unicode scalars. Keep all of them
// until consumed, including across cancelled component start() futures.
#[derive(Debug, Default)]
pub(crate) struct KeyCharCallback {
    pending: RefCell<VecDeque<char>>,
    ready: Callback,
}

impl KeyCharCallback {
    pub fn signal(&self, text: &QString) {
        match String::try_from(text) {
            Ok(text) => {
                if !text.is_empty() {
                    self.pending.borrow_mut().extend(text.chars());
                    self.ready.signal::<GlobalRuntime>(());
                }
            }
            Err(_e) => {
                compio_log::error!("Failed to decode keyboard text: {_e:?}");
            }
        }
    }

    pub async fn wait(&self) -> char {
        loop {
            if let Some(c) = self.pending.borrow_mut().pop_front() {
                return c;
            }
            self.ready.wait().await;
        }
    }
}
