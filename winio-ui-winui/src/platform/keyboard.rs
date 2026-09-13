use std::rc::Rc;

use send_wrapper::SendWrapper;
use windows_core::Interface;
use winio_callback::Callback;
use winio_primitive::KeyCode;
use winio_ui_windows_common::{KeyCharCallback, key_code};
use winui3::Microsoft::UI::Xaml::{FocusState, UIElement};

use crate::{GlobalRuntime, Result};

#[derive(Debug)]
pub(crate) struct Keyboard {
    key_down: SendWrapper<Rc<Callback<KeyCode>>>,
    key_up: SendWrapper<Rc<Callback<KeyCode>>>,
    key_char: SendWrapper<Rc<KeyCharCallback>>,
}

impl Keyboard {
    pub fn new(element: &UIElement) -> Result<Self> {
        element.SetIsTabStop(true)?;
        let key_down = SendWrapper::new(Rc::new(Callback::new()));
        let key_up = SendWrapper::new(Rc::new(Callback::new()));
        let key_char = SendWrapper::new(Rc::new(KeyCharCallback::default()));

        element
            .PointerPressed(|sender, args| {
                let sender = sender.ok()?.cast::<UIElement>()?;
                sender.Focus(FocusState::Programmatic)?;
                let args = args.ok()?;
                args.SetHandled(true)?;
                Ok(())
            })?
            .forget();

        element
            .KeyDown({
                let key_down = key_down.clone();
                move |_, args| {
                    let args = args.ok()?;
                    let code = u16::try_from(args.Key()?.0).map_or(KeyCode::Unidentified, key_code);
                    args.SetHandled(true)?;
                    key_down.signal::<GlobalRuntime>(code);
                    Ok(())
                }
            })?
            .forget();
        element
            .KeyUp({
                let key_up = key_up.clone();
                move |_, args| {
                    let args = args.ok()?;
                    let code = u16::try_from(args.Key()?.0).map_or(KeyCode::Unidentified, key_code);
                    args.SetHandled(true)?;
                    key_up.signal::<GlobalRuntime>(code);
                    Ok(())
                }
            })?
            .forget();
        element
            .CharacterReceived({
                let key_char = key_char.clone();
                move |_, args| {
                    let args = args.ok()?;
                    let unit = args.Character()?;
                    let repeat = args.KeyStatus()?.RepeatCount as usize;
                    args.SetHandled(true)?;
                    key_char.signal_utf16(unit, repeat);
                    Ok(())
                }
            })?
            .forget();

        Ok(Self {
            key_down,
            key_up,
            key_char,
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
