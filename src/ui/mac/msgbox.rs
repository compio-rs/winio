use std::{cell::RefCell, io};

use compio::buf::arrayvec::ArrayVec;
use icrate::{
    block2::ConcreteBlock,
    AppKit::{
        NSAlert, NSAlertFirstButtonReturn, NSAlertStyleCritical, NSAlertStyleInformational,
        NSAlertStyleWarning,
    },
    Foundation::{MainThreadMarker, NSString},
};

use crate::{MessageBoxButton, MessageBoxResponse, MessageBoxStyle, Window};

async fn msgbox_custom(
    parent: Option<&Window>,
    msg: String,
    title: String,
    instr: String,
    style: MessageBoxStyle,
    btns: MessageBoxButton,
) -> io::Result<MessageBoxResponse> {
    unsafe {
        let alert = NSAlert::new(MainThreadMarker::new().unwrap());
        if let Some(parent) = parent {
            alert.window().setParentWindow(Some(&parent.as_nswindow()));
        }
        alert.setAlertStyle(match style {
            MessageBoxStyle::Info => NSAlertStyleInformational,
            MessageBoxStyle::Warning | MessageBoxStyle::Error => NSAlertStyleCritical,
            _ => NSAlertStyleWarning,
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

        if let Some(parent) = parent {
            let (tx, rx) = futures_channel::oneshot::channel();
            let tx = RefCell::new(Some(tx));
            let block = ConcreteBlock::new(|res| {
                tx.borrow_mut()
                    .take()
                    .expect("the handler should only be called once")
                    .send(responses[(res - NSAlertFirstButtonReturn) as usize])
                    .ok();
            });
            alert.beginSheetModalForWindow_completionHandler(&parent.as_nswindow(), Some(&block));
            Ok(rx.await.expect("NSAlert cancelled"))
        } else {
            let res = alert.runModal();
            Ok(responses[res as usize - NSAlertFirstButtonReturn as usize])
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
            btns: MessageBoxButton::Ok,
        }
    }

    pub async fn show(self, parent: Option<&Window>) -> io::Result<MessageBoxResponse> {
        msgbox_custom(
            parent, self.msg, self.title, self.instr, self.style, self.btns,
        )
        .await
    }

    pub fn message(mut self, msg: impl AsRef<str>) -> Self {
        self.msg = msg.as_ref().to_string();
        self
    }

    pub fn title(mut self, title: impl AsRef<str>) -> Self {
        self.title = title.as_ref().to_string();
        self
    }

    pub fn instruction(mut self, instr: impl AsRef<str>) -> Self {
        self.instr = instr.as_ref().to_string();
        self
    }

    pub fn style(mut self, style: MessageBoxStyle) -> Self {
        self.style = style;
        self
    }

    pub fn buttons(mut self, btns: MessageBoxButton) -> Self {
        self.btns = btns;
        self
    }
}
