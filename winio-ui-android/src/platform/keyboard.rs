use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
};

use jni::{
    Env,
    refs::{LoaderContext, Reference},
};
use jni_min_helper::{DynamicProxy, JBoolean};
use ndk_sys as ndk;
use winio_callback::SyncCallback;
use winio_primitive::KeyCode;

use crate::{
    Result,
    java::android::view::{KeyEvent, View, ViewOnKeyListener},
};

pub(crate) fn key_code(env: &mut Env<'_>, event: &KeyEvent<'_>) -> jni::errors::Result<KeyCode> {
    Ok(match event.get_key_code(env)? as u32 {
        ndk::AKEYCODE_DEL => KeyCode::Backspace,
        ndk::AKEYCODE_ENTER | ndk::AKEYCODE_NUMPAD_ENTER => KeyCode::Enter,
        ndk::AKEYCODE_DPAD_LEFT => KeyCode::Left,
        ndk::AKEYCODE_DPAD_RIGHT => KeyCode::Right,
        ndk::AKEYCODE_DPAD_UP => KeyCode::Up,
        ndk::AKEYCODE_DPAD_DOWN => KeyCode::Down,
        ndk::AKEYCODE_MOVE_HOME => KeyCode::Home,
        ndk::AKEYCODE_MOVE_END => KeyCode::End,
        ndk::AKEYCODE_PAGE_UP => KeyCode::PageUp,
        ndk::AKEYCODE_PAGE_DOWN => KeyCode::PageDown,
        ndk::AKEYCODE_TAB => KeyCode::Tab,
        ndk::AKEYCODE_FORWARD_DEL => KeyCode::Delete,
        ndk::AKEYCODE_INSERT => KeyCode::Insert,
        ndk::AKEYCODE_ESCAPE => KeyCode::Esc,
        ndk::AKEYCODE_CAPS_LOCK => KeyCode::CapsLock,
        ndk::AKEYCODE_SCROLL_LOCK => KeyCode::ScrollLock,
        ndk::AKEYCODE_NUM_LOCK => KeyCode::NumLock,
        ndk::AKEYCODE_SYSRQ => KeyCode::PrintScreen,
        ndk::AKEYCODE_BREAK => KeyCode::Pause,
        ndk::AKEYCODE_MENU => KeyCode::Menu,
        ndk::AKEYCODE_CLEAR => KeyCode::Clear,
        ndk::AKEYCODE_SHIFT_LEFT | ndk::AKEYCODE_SHIFT_RIGHT => KeyCode::Shift,
        ndk::AKEYCODE_CTRL_LEFT | ndk::AKEYCODE_CTRL_RIGHT => KeyCode::Control,
        ndk::AKEYCODE_ALT_LEFT | ndk::AKEYCODE_ALT_RIGHT => KeyCode::Alt,
        ndk::AKEYCODE_META_LEFT | ndk::AKEYCODE_META_RIGHT => KeyCode::Super,
        code @ ndk::AKEYCODE_F1..=ndk::AKEYCODE_F12 => {
            KeyCode::F((code - ndk::AKEYCODE_F1 + 1) as u8)
        }
        _ => {
            // Use the device's character map with no modifiers, rather than
            // assuming that Android key codes describe a US keyboard layout.
            let Some(c) = unicode_char(event.get_unicode_char(env, 0)?) else {
                return Ok(KeyCode::Unidentified);
            };
            let mut upper = c.to_uppercase();
            let c = upper.next().unwrap();
            if c.is_control() || upper.next().is_some() {
                KeyCode::Unidentified
            } else {
                KeyCode::Char(c)
            }
        }
    })
}

fn unicode_char(value: i32) -> Option<char> {
    // Zero means no character; dead-key results have a high-bit marker and
    // are not Unicode scalars. Character composition is not handled here.
    char::from_u32(value as u32).filter(|c| *c != '\0')
}

#[derive(Debug)]
pub(crate) struct Keyboard {
    key_down: Arc<SyncCallback<KeyCode>>,
    key_up: Arc<SyncCallback<KeyCode>>,
    key_char: Arc<KeyCharCallback>,
    #[allow(dead_code)]
    key_proxy: DynamicProxy,
}

impl Keyboard {
    pub fn new(env: &mut Env<'_>, view: &View<'_>) -> Result<Self> {
        let key_down = Arc::new(SyncCallback::new());
        let key_up = Arc::new(SyncCallback::new());
        let key_char = Arc::new(KeyCharCallback::default());
        let key_proxy = DynamicProxy::build(
            env,
            &LoaderContext::None,
            [ViewOnKeyListener::class_name()],
            {
                let key_down = key_down.clone();
                let key_up = key_up.clone();
                let key_char = key_char.clone();
                move |env, _method, args| {
                    // onKey(View view, int keyCode, KeyEvent event)
                    let event = args.get_element(env, 2)?;
                    let event = unsafe { KeyEvent::from_raw(env, event.into_raw()) };
                    let action = event.get_action(env)? as u32;
                    if !matches!(
                        action,
                        ndk::AKEY_EVENT_ACTION_DOWN | ndk::AKEY_EVENT_ACTION_UP
                    ) {
                        return Ok(JBoolean::new(env, false)?.into());
                    }
                    let code = key_code(env, &event)?;
                    let mut handled = code != KeyCode::Unidentified;
                    if action == ndk::AKEY_EVENT_ACTION_DOWN {
                        key_down.signal(code);
                        let modifiers = event.get_meta_state(env)?;
                        if modifiers as u32 & ndk::AMETA_META_ON == 0
                            && let Some(c) = unicode_char(event.get_unicode_char(env, modifiers)?)
                        {
                            key_char.signal(c);
                            handled = true;
                        }
                    } else {
                        key_up.signal(code);
                    }
                    // Let Android handle device/system keys outside our portable set.
                    Ok(JBoolean::new(env, handled)?.into())
                }
            },
        )?;
        view.set_focusable_in_touch_mode(env, true)?;
        view.set_on_key_listener(env, &key_proxy)?;
        Ok(Self {
            key_down,
            key_up,
            key_char,
            key_proxy,
        })
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

#[derive(Debug, Default)]
struct KeyCharCallback {
    pending: Mutex<VecDeque<char>>,
    ready: SyncCallback,
}

impl KeyCharCallback {
    fn signal(&self, c: char) {
        self.pending.lock().unwrap().push_back(c);
        self.ready.signal(());
    }

    async fn wait(&self) -> char {
        loop {
            if let Some(c) = self.pending.lock().unwrap().pop_front() {
                return c;
            }
            self.ready.wait().await;
        }
    }
}
