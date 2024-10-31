use std::{cell::RefCell, rc::Rc};

use block2::StackBlock;
use compio::buf::arrayvec::ArrayVec;
use objc2::rc::Id;
use objc2_app_kit::{
    NSAlert, NSAlertFirstButtonReturn, NSAlertStyle, NSImage, NSImageNameCaution, NSImageNameInfo,
};
use objc2_foundation::{MainThreadMarker, NSString, ns_string};

use crate::{AsRawWindow, AsWindow, MessageBoxButton, MessageBoxResponse, MessageBoxStyle};

async fn msgbox_custom(
    parent: Option<impl AsWindow>,
    msg: Id<NSString>,
    title: Id<NSString>,
    instr: Id<NSString>,
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
        let image = match style {
            MessageBoxStyle::Info => NSImage::imageNamed(NSImageNameInfo),
            MessageBoxStyle::Warning | MessageBoxStyle::Error => {
                NSImage::imageNamed(NSImageNameCaution)
            }
            _ => None,
        };
        alert.setIcon(image.as_deref());

        alert.window().setTitle(&title);
        if instr.is_empty() {
            alert.setMessageText(&msg);
        } else {
            alert.setMessageText(&instr);
            alert.setInformativeText(&msg);
        }

        let mut responses = ArrayVec::<MessageBoxResponse, 6>::new();

        if btns.contains(MessageBoxButton::Ok) {
            alert.addButtonWithTitle(ns_string!("Ok"));
            responses.push(MessageBoxResponse::Ok);
        }
        if btns.contains(MessageBoxButton::Yes) {
            alert.addButtonWithTitle(ns_string!("Yes"));
            responses.push(MessageBoxResponse::Yes);
        }
        if btns.contains(MessageBoxButton::No) {
            alert.addButtonWithTitle(ns_string!("No"));
            responses.push(MessageBoxResponse::No);
        }
        if btns.contains(MessageBoxButton::Cancel) {
            alert.addButtonWithTitle(ns_string!("Cancel"));
            responses.push(MessageBoxResponse::Cancel);
        }
        if btns.contains(MessageBoxButton::Retry) {
            alert.addButtonWithTitle(ns_string!("Try again"));
            responses.push(MessageBoxResponse::Retry);
        }
        if btns.contains(MessageBoxButton::Close) {
            alert.addButtonWithTitle(ns_string!("Close"));
            responses.push(MessageBoxResponse::Close);
        }

        for b in cbtns {
            alert.addButtonWithTitle(&b.text);
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
            let res = rx.await.expect("NSAlert cancelled");
            parent.makeKeyWindow();
            res
        } else {
            let res = alert.runModal();
            responses[res as usize - NSAlertFirstButtonReturn as usize]
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct MessageBox {
    msg: Id<NSString>,
    title: Id<NSString>,
    instr: Id<NSString>,
    style: MessageBoxStyle,
    btns: MessageBoxButton,
    cbtns: Vec<CustomButton>,
}

impl MessageBox {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn show(self, parent: Option<impl AsWindow>) -> MessageBoxResponse {
        msgbox_custom(
            parent, self.msg, self.title, self.instr, self.style, self.btns, self.cbtns,
        )
        .await
    }

    pub fn message(&mut self, msg: &str) {
        self.msg = NSString::from_str(msg);
    }

    pub fn title(&mut self, title: &str) {
        self.title = NSString::from_str(title);
    }

    pub fn instruction(&mut self, instr: &str) {
        self.instr = NSString::from_str(instr);
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
    pub text: Id<NSString>,
}

impl CustomButton {
    pub fn new(result: u16, text: &str) -> Self {
        Self {
            result,
            text: NSString::from_str(text),
        }
    }
}
