use std::{cell::RefCell, rc::Rc};

use block2::StackBlock;
use compio::buf::arrayvec::ArrayVec;
use objc2_app_kit::{NSAlert, NSAlertFirstButtonReturn, NSAlertStyle};
use objc2_foundation::{MainThreadMarker, NSString};

use crate::{AsRawWindow, AsWindow, MessageBoxButton, MessageBoxResponse, MessageBoxStyle};

async fn msgbox_custom(
    parent: Option<impl AsWindow>,
    msg: String,
    title: String,
    instr: String,
    style: MessageBoxStyle,
    btns: MessageBoxButton,
    cbtns: Vec<CustomButton>,
) -> MessageBoxResponse {
    unsafe {
        let parent = parent.map(|p| p.as_window().as_raw_window());

        let alert = NSAlert::new(MainThreadMarker::new().unwrap());
        if let Some(parent) = &parent {
            alert.window().setParentWindow(Some(parent));
        }
        alert.setAlertStyle(match style {
            MessageBoxStyle::Info => NSAlertStyle::Informational,
            MessageBoxStyle::Warning | MessageBoxStyle::Error => NSAlertStyle::Critical,
            _ => NSAlertStyle::Warning,
        });

        alert.window().setTitle(&NSString::from_str(&title));
        if instr.is_empty() {
            alert.setMessageText(&NSString::from_str(&msg));
        } else {
            alert.setMessageText(&NSString::from_str(&instr));
            alert.setInformativeText(&NSString::from_str(&msg));
        }

        let mut responses = ArrayVec::<MessageBoxResponse, 6>::new();

        if btns.contains(MessageBoxButton::Ok) {
            alert.addButtonWithTitle(&NSString::from_str("Ok"));
            responses.push(MessageBoxResponse::Ok);
        }
        if btns.contains(MessageBoxButton::Yes) {
            alert.addButtonWithTitle(&NSString::from_str("Yes"));
            responses.push(MessageBoxResponse::Yes);
        }
        if btns.contains(MessageBoxButton::No) {
            alert.addButtonWithTitle(&NSString::from_str("No"));
            responses.push(MessageBoxResponse::No);
        }
        if btns.contains(MessageBoxButton::Cancel) {
            alert.addButtonWithTitle(&NSString::from_str("Cancel"));
            responses.push(MessageBoxResponse::Cancel);
        }
        if btns.contains(MessageBoxButton::Retry) {
            alert.addButtonWithTitle(&NSString::from_str("Try again"));
            responses.push(MessageBoxResponse::Retry);
        }
        if btns.contains(MessageBoxButton::Close) {
            alert.addButtonWithTitle(&NSString::from_str("Close"));
            responses.push(MessageBoxResponse::Close);
        }

        for b in cbtns {
            alert.addButtonWithTitle(&NSString::from_str(&b.text));
            responses.push(MessageBoxResponse::Custom(b.result));
        }

        if let Some(parent) = &parent {
            let (tx, rx) = futures_channel::oneshot::channel();
            let tx = Rc::new(RefCell::new(Some(tx)));
            let block = StackBlock::new(move |res| {
                tx.borrow_mut()
                    .take()
                    .expect("the handler should only be called once")
                    .send(responses[(res - NSAlertFirstButtonReturn) as usize])
                    .ok();
            });
            alert.beginSheetModalForWindow_completionHandler(parent, Some(&block));
            rx.await.expect("NSAlert cancelled")
        } else {
            let res = alert.runModal();
            responses[res as usize - NSAlertFirstButtonReturn as usize]
        }
    }
}

#[derive(Debug, Clone)]
pub struct MessageBox {
    msg: String,
    title: String,
    instr: String,
    style: MessageBoxStyle,
    btns: MessageBoxButton,
    cbtns: Vec<CustomButton>,
}

impl Default for MessageBox {
    fn default() -> Self {
        Self::new()
    }
}

impl MessageBox {
    pub fn new() -> Self {
        Self {
            msg: String::new(),
            title: String::new(),
            instr: String::new(),
            style: MessageBoxStyle::None,
            btns: MessageBoxButton::None,
            cbtns: vec![],
        }
    }

    pub async fn show(self, parent: Option<impl AsWindow>) -> MessageBoxResponse {
        msgbox_custom(
            parent, self.msg, self.title, self.instr, self.style, self.btns, self.cbtns,
        )
        .await
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
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct CustomButton {
    pub result: u16,
    pub text: String,
}

impl CustomButton {
    pub fn new(result: u16, text: &str) -> Self {
        Self {
            result,
            text: text.to_string(),
        }
    }
}
