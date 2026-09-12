use std::{cell::RefCell, collections::VecDeque, rc::Rc};

use gtk4::{
    EventControllerKey,
    gdk::{Key, ModifierType},
    glib::{Propagation, translate::IntoGlib},
    prelude::*,
};
use winio_callback::Callback;
use winio_primitive::KeyCode;

use crate::GlobalRuntime;

pub(crate) fn key_code(key: Key) -> KeyCode {
    match key {
        Key::BackSpace => KeyCode::Backspace,
        Key::Return | Key::ISO_Enter | Key::KP_Enter => KeyCode::Enter,
        Key::Left | Key::KP_Left => KeyCode::Left,
        Key::Right | Key::KP_Right => KeyCode::Right,
        Key::Up | Key::KP_Up => KeyCode::Up,
        Key::Down | Key::KP_Down => KeyCode::Down,
        Key::Home | Key::KP_Home => KeyCode::Home,
        Key::End | Key::KP_End => KeyCode::End,
        Key::Page_Up | Key::KP_Page_Up => KeyCode::PageUp,
        Key::Page_Down | Key::KP_Page_Down => KeyCode::PageDown,
        Key::Tab | Key::ISO_Left_Tab | Key::KP_Tab => KeyCode::Tab,
        Key::Delete | Key::KP_Delete => KeyCode::Delete,
        Key::Insert | Key::KP_Insert => KeyCode::Insert,
        Key::Escape => KeyCode::Esc,
        Key::Caps_Lock => KeyCode::CapsLock,
        Key::Scroll_Lock => KeyCode::ScrollLock,
        Key::Num_Lock => KeyCode::NumLock,
        Key::Print | Key::Sys_Req => KeyCode::PrintScreen,
        Key::Pause | Key::Break => KeyCode::Pause,
        Key::Menu => KeyCode::Menu,
        Key::Clear | Key::Begin | Key::KP_Begin => KeyCode::Clear,
        Key::Shift_L | Key::Shift_R => KeyCode::Shift,
        Key::Control_L | Key::Control_R => KeyCode::Control,
        Key::Alt_L | Key::Alt_R => KeyCode::Alt,
        Key::ISO_Level3_Shift | Key::Mode_switch => KeyCode::AltGr,
        Key::Super_L | Key::Super_R => KeyCode::Super,
        Key::Hyper_L | Key::Hyper_R => KeyCode::Hyper,
        Key::Meta_L | Key::Meta_R => KeyCode::Meta,
        key if (Key::F1..=Key::F35).contains(&key) => {
            KeyCode::F((key.into_glib() - Key::F1.into_glib() + 1) as u8)
        }
        key if (Key::KP_F1..=Key::KP_F4).contains(&key) => {
            KeyCode::F((key.into_glib() - Key::KP_F1.into_glib() + 1) as u8)
        }
        _ => key
            .to_upper()
            .to_unicode()
            .filter(|c| !c.is_control())
            .and_then(|c| u8::try_from(c as u32).ok())
            .map_or(KeyCode::Unidentified, KeyCode::Char),
    }
}

// Translate key symbols directly, without treating shortcuts as text.
fn key_char(key: Key, modifiers: ModifierType) -> Option<char> {
    if modifiers.intersects(
        ModifierType::CONTROL_MASK
            | ModifierType::ALT_MASK
            | ModifierType::SUPER_MASK
            | ModifierType::HYPER_MASK
            | ModifierType::META_MASK,
    ) {
        None
    } else {
        key.to_unicode()
    }
}

#[derive(Debug)]
pub(crate) struct Keyboard {
    on_key_down: Rc<Callback<KeyCode>>,
    on_key_up: Rc<Callback<KeyCode>>,
    on_key_char: Rc<KeyCharCallback>,
}

impl Keyboard {
    pub fn new(widget: &impl IsA<gtk4::Widget>) -> Self {
        let on_key_down = Rc::new(Callback::new());
        let on_key_up = Rc::new(Callback::new());
        let on_key_char = Rc::new(KeyCharCallback::default());

        let controller = EventControllerKey::new();
        controller.connect_key_pressed({
            let on_key_down = on_key_down.clone();
            let on_key_char = on_key_char.clone();
            move |_, key, _, modifiers| {
                on_key_down.signal::<GlobalRuntime>(key_code(key));
                if let Some(c) = key_char(key, modifiers) {
                    on_key_char.signal_char(c);
                }
                Propagation::Stop
            }
        });
        controller.connect_key_released({
            let on_key_up = on_key_up.clone();
            move |_, key, _, _| {
                on_key_up.signal::<GlobalRuntime>(key_code(key));
            }
        });
        widget.add_controller(controller);
        widget.set_focusable(true);

        Self {
            on_key_down,
            on_key_up,
            on_key_char,
        }
    }

    pub async fn wait_key_down(&self) -> KeyCode {
        self.on_key_down.wait().await
    }

    pub async fn wait_key_up(&self) -> KeyCode {
        self.on_key_up.wait().await
    }

    pub async fn wait_key_char(&self) -> char {
        self.on_key_char.wait().await
    }
}

// Keep every input character, even if the component is busy or its
// start() future is cancelled while processing an earlier character.
#[derive(Debug, Default)]
struct KeyCharCallback {
    pending: RefCell<VecDeque<char>>,
    ready: Callback,
}

impl KeyCharCallback {
    fn signal_char(&self, c: char) {
        self.pending.borrow_mut().push_back(c);
        self.ready.signal::<GlobalRuntime>(());
    }

    async fn wait(&self) -> char {
        loop {
            if let Some(c) = self.pending.borrow_mut().pop_front() {
                return c;
            }
            self.ready.wait().await;
        }
    }
}
