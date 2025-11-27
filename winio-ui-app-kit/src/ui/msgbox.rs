use std::{cell::Cell, rc::Rc};

use block2::StackBlock;
use compio::arrayvec::ArrayVec;
use objc2::{MainThreadOnly, rc::Retained};
use objc2_app_kit::{
    NSAlert, NSAlertFirstButtonReturn, NSAlertStyle, NSImage, NSImageNameCaution, NSImageNameInfo,
};
use objc2_foundation::{MainThreadMarker, NSString, ns_string};
use winio_handle::{AsRawWindow, AsWindow};
use winio_primitive::{MessageBoxButton, MessageBoxResponse, MessageBoxStyle};

use crate::{Error, Result, catch};

async fn msgbox_custom(
    parent: Option<impl AsWindow>,
    msg: Retained<NSString>,
    title: Retained<NSString>,
    instr: Retained<NSString>,
    style: MessageBoxStyle,
    btns: MessageBoxButton,
    cbtns: Vec<CustomButton>,
) -> Result<MessageBoxResponse> {
    let parent = parent.map(|p| p.as_window().as_raw_window());
    let mtm = parent
        .as_ref()
        .map(|w| w.mtm())
        .or_else(MainThreadMarker::new)
        .ok_or(Error::NotMainThread)?;

    let (alert, responses) = catch(|| {
        let alert = NSAlert::new(mtm);
        if let Some(parent) = &parent {
            unsafe {
                alert.window().setParentWindow(Some(parent));
            }
        }
        alert.setAlertStyle(match style {
            MessageBoxStyle::Info => NSAlertStyle::Informational,
            MessageBoxStyle::Warning | MessageBoxStyle::Error => NSAlertStyle::Critical,
            _ => NSAlertStyle::Warning,
        });
        let image = match style {
            MessageBoxStyle::Info => NSImage::imageNamed(unsafe { NSImageNameInfo }),
            MessageBoxStyle::Warning | MessageBoxStyle::Error => {
                NSImage::imageNamed(unsafe { NSImageNameCaution })
            }
            _ => None,
        };
        unsafe {
            alert.setIcon(image.as_deref());
        }

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
        Ok((alert, responses))
    })
    .flatten()?;

    let res = if let Some(parent) = &parent {
        let (tx, rx) = local_sync::oneshot::channel();
        let tx = Rc::new(Cell::new(Some(tx)));
        let block = StackBlock::new(move |res| {
            tx.take()
                .expect("the handler should only be called once")
                .send(responses[(res - NSAlertFirstButtonReturn) as usize])
                .ok();
        });
        catch(|| alert.beginSheetModalForWindow_completionHandler(parent, Some(&block)))?;
        let res = rx.await?;
        catch(|| parent.makeKeyWindow())?;
        res
    } else {
        let res = catch(|| alert.runModal())?;
        responses[res as usize - NSAlertFirstButtonReturn as usize]
    };
    Ok(res)
}

#[derive(Debug, Default, Clone)]
pub struct MessageBox {
    msg: Retained<NSString>,
    title: Retained<NSString>,
    instr: Retained<NSString>,
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
