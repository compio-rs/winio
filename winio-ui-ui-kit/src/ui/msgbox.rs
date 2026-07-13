use std::{cell::Cell, rc::Rc};

use block2::StackBlock;
use futures_util::TryFutureExt;
use objc2::rc::Retained;
use objc2_foundation::{MainThreadMarker, NSString, ns_string};
use objc2_ui_kit::{UIAlertAction, UIAlertActionStyle, UIAlertController, UIAlertControllerStyle};
use winio_handle::AsWindow;
use winio_primitive::{MessageBoxButton, MessageBoxResponse, MessageBoxStyle};

use crate::{Error, Result, catch, first_ui_window_scene};

fn msgbox_custom(
    parent: Option<impl AsWindow>,
    msg: String,
    title: String,
    instr: String,
    _style: MessageBoxStyle,
    btns: MessageBoxButton,
    cbtns: Vec<CustomButton>,
) -> Result<impl Future<Output = Result<MessageBoxResponse>> + 'static> {
    let mtm = MainThreadMarker::new().ok_or(Error::NotMainThread)?;

    let (tx, rx) = local_sync::oneshot::channel();
    let tx = Rc::new(Cell::new(Some(tx)));

    catch(|| {
        let alert = UIAlertController::alertControllerWithTitle_message_preferredStyle(
            Some(&NSString::from_str(&title)),
            Some(&NSString::from_str(&if instr.is_empty() {
                msg
            } else {
                format!("{}\n\n{}", instr, msg)
            })),
            UIAlertControllerStyle::Alert,
            mtm,
        );

        let add_action = |response: MessageBoxResponse,
                          text: &NSString,
                          action: UIAlertActionStyle| {
            let tx = tx.clone();
            let block = StackBlock::new(move |_| {
                if let Some(tx) = tx.take() {
                    tx.send(response).ok();
                }
            });
            let action =
                UIAlertAction::actionWithTitle_style_handler(Some(text), action, Some(&block), mtm);
            alert.addAction(&action);
        };

        if btns.contains(MessageBoxButton::Ok) {
            add_action(
                MessageBoxResponse::Ok,
                ns_string!("Ok"),
                UIAlertActionStyle::Default,
            );
        }

        if btns.contains(MessageBoxButton::Yes) {
            add_action(
                MessageBoxResponse::Yes,
                ns_string!("Yes"),
                UIAlertActionStyle::Default,
            );
        }

        if btns.contains(MessageBoxButton::No) {
            add_action(
                MessageBoxResponse::No,
                ns_string!("No"),
                UIAlertActionStyle::Default,
            );
        }

        if btns.contains(MessageBoxButton::Cancel) {
            add_action(
                MessageBoxResponse::Cancel,
                ns_string!("Cancel"),
                UIAlertActionStyle::Cancel,
            );
        }

        if btns.contains(MessageBoxButton::Retry) {
            add_action(
                MessageBoxResponse::Retry,
                ns_string!("Try again"),
                UIAlertActionStyle::Default,
            );
        }

        if btns.contains(MessageBoxButton::Close) {
            add_action(
                MessageBoxResponse::Close,
                ns_string!("Close"),
                UIAlertActionStyle::Destructive,
            );
        }

        for b in cbtns {
            add_action(
                MessageBoxResponse::Custom(b.result),
                &b.text,
                UIAlertActionStyle::Default,
            );
        }

        let controller = if let Some(parent) = parent {
            parent.as_window().as_ui_kit().rootViewController()
        } else {
            first_ui_window_scene()?
                .and_then(|scene| scene.keyWindow())
                .and_then(|wnd| wnd.rootViewController())
        };
        if let Some(vc) = controller {
            vc.presentViewController_animated_completion(&alert, true, None);
        }
        Ok(())
    })
    .flatten()?;

    Ok(rx.map_err(Into::into))
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

    pub fn show(
        self,
        parent: Option<impl AsWindow>,
    ) -> Result<impl Future<Output = Result<MessageBoxResponse>> + 'static> {
        msgbox_custom(
            parent, self.msg, self.title, self.instr, self.style, self.btns, self.cbtns,
        )
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
    pub text: Retained<NSString>,
}

impl CustomButton {
    pub fn new(result: u16, text: &str) -> Self {
        Self {
            result,
            text: NSString::from_str(text),
        }
    }
}
