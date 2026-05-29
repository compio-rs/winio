use jni::{
    Env,
    objects::{JObject, JString},
    refs::LoaderContext,
};
use jni_min_helper::{DynamicProxy, JInteger};
use winio_handle::AsWindow;
use winio_primitive::{MessageBoxButton, MessageBoxResponse, MessageBoxStyle};

use crate::{Error, Result, vm_exec};

jni::bind_java_type! {
    Context => android.content.Context,
}

jni::bind_java_type! {
    AlertDialog => android.app.AlertDialog,
    type_map {
        OnCancelListener => "android.content.DialogInterface$OnCancelListener",
        OnDismissListener => "android.content.DialogInterface$OnDismissListener",
    },
    methods {
        fn show(),
        fn set_on_cancel_listener(listener: &OnCancelListener),
        fn set_on_dismiss_listener(listener: &OnDismissListener),
    }
}

jni::bind_java_type! {
    AlertDialogBuilder => "android.app.AlertDialog$Builder",
    type_map {
        AlertDialog => android.app.AlertDialog,
        Context => android.content.Context,
        OnClickListener => "android.content.DialogInterface$OnClickListener",
    },
    constructors {
        fn new(context: &Context),
    },
    methods {
        fn create() -> AlertDialog,
        fn set_message(message: &JCharSequence) -> AlertDialogBuilder,
        fn set_title(title: &JCharSequence) -> AlertDialogBuilder,
        fn set_positive_button(text: &JCharSequence, listener: &OnClickListener) -> AlertDialogBuilder,
        fn set_negative_button(text: &JCharSequence, listener: &OnClickListener) -> AlertDialogBuilder,
        fn set_neutral_button(text: &JCharSequence, listener: &OnClickListener) -> AlertDialogBuilder,
    }
}

jni::bind_java_type! {
    OnClickListener => "android.content.DialogInterface$OnClickListener",
}

jni::bind_java_type! {
    OnCancelListener => "android.content.DialogInterface$OnCancelListener",
}

jni::bind_java_type! {
    OnDismissListener => "android.content.DialogInterface$OnDismissListener",
}

#[derive(Debug, Default, Clone)]
pub struct MessageBox {
    msg: String,
    title: String,
    instr: String,
    style: MessageBoxStyle,
    btns: MessageBoxButton,
    cbtns: Vec<CustomButton>,
}

impl MessageBox {
    pub fn new() -> Self {
        Self {
            msg: String::new(),
            title: String::new(),
            instr: String::new(),
            style: MessageBoxStyle::None,
            btns: MessageBoxButton::empty(),
            cbtns: vec![],
        }
    }

    pub fn message(&mut self, msg: &str) {
        self.msg = msg.to_string();
    }

    pub fn title(&mut self, title: &str) {
        self.title = title.to_string();
    }

    pub fn instruction(&mut self, instr: &str) {
        self.instr = instr.to_string();
    }

    pub fn style(&mut self, style: MessageBoxStyle) {
        self.style = style;
    }

    pub fn buttons(&mut self, btns: MessageBoxButton) {
        self.btns = btns;
    }

    pub fn custom_button(&mut self, btn: CustomButton) {
        self.cbtns.push(btn);
    }

    pub fn custom_buttons(&mut self, btn: impl IntoIterator<Item = CustomButton>) {
        self.cbtns.extend(btn);
    }

    fn texts_and_responses<'local>(
        env: &mut Env<'local>,
        btns: MessageBoxButton,
        cbtns: &[CustomButton],
    ) -> Result<([Option<MessageBoxResponse>; 3], [JString<'local>; 3])> {
        let mut responses = [None; 3];
        let mut texts = [JString::null(), JString::null(), JString::null()];

        const POSITIVE: usize = 0;
        const NEGATIVE: usize = 1;
        const NEUTRAL: usize = 2;

        let mut push = |response: MessageBoxResponse, text: &str, index: usize| {
            if responses[index].replace(response).is_some() {
                return Err(Error::NotSupported);
            }
            texts[index] = env.new_string(text)?;
            Ok(())
        };
        if btns.contains(MessageBoxButton::Ok) {
            push(MessageBoxResponse::Ok, "Ok", POSITIVE)?;
        }
        if btns.contains(MessageBoxButton::Cancel) {
            push(MessageBoxResponse::Cancel, "Cancel", NEGATIVE)?;
        }
        if btns.contains(MessageBoxButton::Yes) {
            push(MessageBoxResponse::Yes, "Yes", POSITIVE)?;
        }
        if btns.contains(MessageBoxButton::No) {
            push(MessageBoxResponse::No, "No", NEGATIVE)?;
        }
        if btns.contains(MessageBoxButton::Retry) {
            push(MessageBoxResponse::Retry, "Retry", POSITIVE)?;
        }
        if btns.contains(MessageBoxButton::Close) {
            push(MessageBoxResponse::Close, "Close", NEGATIVE)?;
        }
        for cbtn in cbtns {
            push(
                MessageBoxResponse::Custom(cbtn.result),
                &cbtn.label,
                NEUTRAL,
            )?;
        }
        Ok((responses, texts))
    }

    pub async fn show(self, parent: Option<impl AsWindow>) -> Result<MessageBoxResponse> {
        let (mut rx, _proxy) = vm_exec(|env| {
            let activity = if let Some(parent) = parent {
                env.new_local_ref(parent.as_window().to_android())?
            } else {
                env.new_local_ref(crate::current_activity()?)?
            };
            let activity = unsafe { Context::from_raw(env, activity.into_raw()) };
            let builder = AlertDialogBuilder::new(env, &activity)?;
            let msg = if self.instr.is_empty() {
                env.new_string(self.msg)?
            } else {
                env.new_string(format!("{}\n\n{}", self.instr, self.msg))?
            };
            builder.set_message(env, &msg)?;
            if !self.title.is_empty() {
                let title = env.new_string(self.title)?;
                builder.set_title(env, &title)?;
            }
            let (responses, texts) = Self::texts_and_responses(env, self.btns, &self.cbtns)?;

            let (tx, rx) = futures_channel::mpsc::unbounded::<MessageBoxResponse>();
            let proxy = DynamicProxy::build(
                env,
                &LoaderContext::None,
                [
                    jni::jni_str!("android/content/DialogInterface$OnClickListener"),
                    jni::jni_str!("android/content/DialogInterface$OnCancelListener"),
                    jni::jni_str!("android/content/DialogInterface$OnDismissListener"),
                ],
                move |env, method, args| {
                    let name = method.get_name(env)?.try_to_string(env)?;
                    if name == "onCancel" {
                        tx.unbounded_send(MessageBoxResponse::Cancel).ok();
                    } else if name == "onDismiss" {
                        tx.unbounded_send(MessageBoxResponse::Close).ok();
                    } else if name == "onClick" {
                        let which = args.get_element(env, 1)?;
                        let which =
                            unsafe { JInteger::from_raw(env, which.into_raw()) }.value(env)?;
                        let response = match which {
                            -3 => responses[2].unwrap(),
                            -2 => responses[1].unwrap(),
                            -1 => responses[0].unwrap(),
                            _ => panic!("Unknown button index: {}", which),
                        };
                        tx.unbounded_send(response).ok();
                    }
                    Ok(JObject::null())
                },
            )?;

            let click_listener = env.new_local_ref(proxy.as_ref())?;
            let click_listener =
                unsafe { OnClickListener::from_raw(env, click_listener.into_raw()) };
            if responses[0].is_some() {
                builder.set_positive_button(env, &texts[0], &click_listener)?;
            }
            if responses[1].is_some() {
                builder.set_negative_button(env, &texts[1], &click_listener)?;
            }
            if responses[2].is_some() {
                builder.set_neutral_button(env, &texts[2], &click_listener)?;
            }

            let dialog = builder.create(env)?;
            let cancel_listener =
                unsafe { OnCancelListener::from_raw(env, click_listener.into_raw()) };
            dialog.set_on_cancel_listener(env, &cancel_listener)?;
            let dismiss_listener =
                unsafe { OnDismissListener::from_raw(env, cancel_listener.into_raw()) };
            dialog.set_on_dismiss_listener(env, &dismiss_listener)?;
            dialog.show(env)?;
            Ok((rx, proxy))
        })?;
        rx.recv()
            .await
            .map_err(|e| Error::Io(std::io::Error::other(e)))
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct CustomButton {
    result: u16,
    label: String,
}

impl CustomButton {
    pub fn new(result: u16, label: impl AsRef<str>) -> Self {
        Self {
            result,
            label: label.as_ref().to_string(),
        }
    }
}
