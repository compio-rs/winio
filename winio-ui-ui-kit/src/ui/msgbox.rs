use std::{cell::Cell, rc::Rc};

use arrayvec::ArrayVec;
use block2::StackBlock;
use objc2::rc::Retained;
use objc2_foundation::{MainThreadMarker, NSString};
use objc2_ui_kit::{
    UIAlertAction, UIAlertActionStyle, UIAlertController, UIAlertControllerStyle, UIApplication,
};
use winio_handle::AsWindow;
use winio_primitive::{MessageBoxButton, MessageBoxResponse, MessageBoxStyle};

use crate::{Error, Result, catch};

async fn msgbox_custom(
    parent: Option<impl AsWindow>,
    msg: String,
    title: String,
    instr: String,
    style: MessageBoxStyle,
    btns: MessageBoxButton,
    cbtns: Vec<CustomButton>,
) -> Result<MessageBoxResponse> {
    let mtm = MainThreadMarker::new().ok_or(Error::NotMainThread)?;

    let (tx, rx) = local_sync::oneshot::channel();
    let tx = Rc::new(Cell::new(Some(tx)));

    catch(|| {
        let alert = UIAlertController::alertControllerWithTitle_message_preferredStyle(
            Some(&NSString::from_str(&title)),
            Some(&NSString::from_str(&msg)),
            UIAlertControllerStyle::Alert,
            mtm,
        );

        let mut responses = ArrayVec::<MessageBoxResponse, 6>::new();

        if btns.contains(MessageBoxButton::Ok) {
            let tx = tx.clone();
            let block = StackBlock::new(move |_| {
                if let Some(tx) = tx.take() {
                    tx.send(MessageBoxResponse::Ok).ok();
                }
            });
            let action = UIAlertAction::actionWithTitle_style_handler(
                Some(ns_string!("Ok")),
                UIAlertActionStyle::Default,
                Some(&block),
                mtm,
            );
            alert.addAction(&action);
            responses.push(MessageBoxResponse::Ok);
        }

        if btns.contains(MessageBoxButton::Yes) {
            let tx = tx.clone();
            let block = StackBlock::new(move |_| {
                if let Some(tx) = tx.take() {
                    tx.send(MessageBoxResponse::Yes).ok();
                }
            });
            let action = UIAlertAction::actionWithTitle_style_handler(
                Some(ns_string!("Yes")),
                UIAlertActionStyle::Default,
                Some(&block),
                mtm,
            );
            alert.addAction(&action);
            responses.push(MessageBoxResponse::Yes);
        }

        if btns.contains(MessageBoxButton::No) {
            let tx = tx.clone();
            let block = StackBlock::new(move |_| {
                if let Some(tx) = tx.take() {
                    tx.send(MessageBoxResponse::No).ok();
                }
            });
            let action = UIAlertAction::actionWithTitle_style_handler(
                Some(ns_string!("No")),
                UIAlertActionStyle::Default,
                Some(&block),
                mtm,
            );
            alert.addAction(&action);
            responses.push(MessageBoxResponse::No);
        }

        if btns.contains(MessageBoxButton::Cancel) {
            let tx = tx.clone();
            let block = StackBlock::new(move |_| {
                if let Some(tx) = tx.take() {
                    tx.send(MessageBoxResponse::Cancel).ok();
                }
            });
            let action = UIAlertAction::actionWithTitle_style_handler(
                Some(ns_string!("Cancel")),
                UIAlertActionStyle::Cancel,
                Some(&block),
                mtm,
            );
            alert.addAction(&action);
            responses.push(MessageBoxResponse::Cancel);
        }

        if btns.contains(MessageBoxButton::Retry) {
            let tx = tx.clone();
            let block = StackBlock::new(move |_| {
                if let Some(tx) = tx.take() {
                    tx.send(MessageBoxResponse::Retry).ok();
                }
            });
            let action = UIAlertAction::actionWithTitle_style_handler(
                Some(ns_string!("Try again")),
                UIAlertActionStyle::Default,
                Some(&block),
                mtm,
            );
            alert.addAction(&action);
            responses.push(MessageBoxResponse::Retry);
        }

        if btns.contains(MessageBoxButton::Close) {
            let tx = tx.clone();
            let block = StackBlock::new(move |_| {
                if let Some(tx) = tx.take() {
                    tx.send(MessageBoxResponse::Close).ok();
                }
            });
            let action = UIAlertAction::actionWithTitle_style_handler(
                Some(ns_string!("Close")),
                UIAlertActionStyle::Default,
                Some(&block),
                mtm,
            );
            alert.addAction(&action);
            responses.push(MessageBoxResponse::Close);
        }

        for b in cbtns {
            let result = b.result;
            let tx = tx.clone();
            let block = StackBlock::new(move |_| {
                if let Some(tx) = tx.take() {
                    tx.send(MessageBoxResponse::Custom(result)).ok();
                }
            });
            let action = UIAlertAction::actionWithTitle_style_handler(
                Some(&b.text),
                UIAlertActionStyle::Default,
                Some(&block),
                mtm,
            );
            alert.addAction(&action);
            responses.push(MessageBoxResponse::Custom(result));
        }

        // Show the alert
        let app = UIApplication::sharedApplication(mtm);
        if let Some(key_window) = app.keyWindow()
            && let Some(vc) = key_window.rootViewController()
        {
            vc.presentViewController_animated_completion(&alert, true, None);
        }
    })?;

    rx.await.map_err(Into::into)
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

// SAFETY: NSString is thread-safe.
unsafe impl Send for MessageBox {}
unsafe impl Sync for MessageBox {}

impl MessageBox {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn show(self, parent: Option<impl AsWindow>) -> Result<MessageBoxResponse> {
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
    pub text: Retained<objc2_foundation::NSString>,
}

impl CustomButton {
    pub fn new(result: u16, text: &str) -> Self {
        Self {
            result,
            text: objc2_foundation::NSString::from_str(text),
        }
    }
}

use objc2_foundation::ns_string;
