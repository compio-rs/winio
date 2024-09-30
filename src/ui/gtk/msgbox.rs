#![allow(deprecated)]

use std::{cell::RefCell, io};

use gtk4::prelude::{DialogExt, GtkWindowExt, WidgetExt};

use crate::{MessageBoxButton, MessageBoxResponse, MessageBoxStyle, Window};

const CUSTOM_CANCEL: u16 = 0;
const CUSTOM_NO: u16 = 1;
const CUSTOM_OK: u16 = 2;
const CUSTOM_RETRY: u16 = 3;
const CUSTOM_YES: u16 = 4;
const CUSTOM_CLOSE: u16 = 5;

async fn msgbox_custom(
    parent: Option<&Window>,
    msg: String,
    title: String,
    instr: String,
    style: MessageBoxStyle,
    btns: MessageBoxButton,
    cbtns: Vec<CustomButton>,
) -> io::Result<MessageBoxResponse> {
    let default_btns = match btns as i32 {
        // Ok
        1 => gtk4::ButtonsType::Ok,
        // Close
        32 => gtk4::ButtonsType::Close,
        // Cancel
        8 => gtk4::ButtonsType::Cancel,
        // Yes | No
        6 => gtk4::ButtonsType::YesNo,
        // Ok | Cancel
        9 => gtk4::ButtonsType::OkCancel,
        // others
        _ => gtk4::ButtonsType::None,
    };

    let dialog = gtk4::MessageDialog::new(
        parent.map(|w| w.as_window()),
        gtk4::DialogFlags::DESTROY_WITH_PARENT | gtk4::DialogFlags::MODAL,
        match style {
            MessageBoxStyle::Info => gtk4::MessageType::Info,
            MessageBoxStyle::Warning => gtk4::MessageType::Warning,
            MessageBoxStyle::Error => gtk4::MessageType::Error,
            _ => gtk4::MessageType::Other,
        },
        default_btns,
        if instr.is_empty() { &msg } else { &instr },
    );
    dialog.set_title(Some(&title));
    dialog.set_secondary_text(if instr.is_empty() { None } else { Some(&msg) });

    if default_btns == gtk4::ButtonsType::None {
        if btns.contains(MessageBoxButton::Ok) {
            dialog.add_button("Ok", gtk4::ResponseType::Other(CUSTOM_OK));
        }
        if btns.contains(MessageBoxButton::Yes) {
            dialog.add_button("Yes", gtk4::ResponseType::Other(CUSTOM_YES));
        }
        if btns.contains(MessageBoxButton::No) {
            dialog.add_button("No", gtk4::ResponseType::Other(CUSTOM_NO));
        }
        if btns.contains(MessageBoxButton::Cancel) {
            dialog.add_button("Cancel", gtk4::ResponseType::Other(CUSTOM_CANCEL));
        }
        if btns.contains(MessageBoxButton::Retry) {
            dialog.add_button("Retry", gtk4::ResponseType::Other(CUSTOM_RETRY));
        }
        if btns.contains(MessageBoxButton::Close) {
            dialog.add_button("Close", gtk4::ResponseType::Other(CUSTOM_CLOSE));
        }
    }
    for b in cbtns {
        dialog.add_button(&b.text, gtk4::ResponseType::Other(b.result));
    }

    let (tx, rx) = futures_channel::oneshot::channel();
    dialog.connect_response({
        let tx = RefCell::new(Some(tx));
        move |dialog, res| {
            let tx = tx.borrow_mut().take();
            if let Some(tx) = tx {
                tx.send(res).ok();
                dialog.close();
            }
        }
    });
    dialog.set_visible(true);
    let res = rx
        .await
        .map_err(|e| io::Error::new(io::ErrorKind::ConnectionAborted, e))?;

    let res = match res {
        gtk4::ResponseType::Ok => MessageBoxResponse::Ok,
        gtk4::ResponseType::Cancel => MessageBoxResponse::Cancel,
        gtk4::ResponseType::Close => MessageBoxResponse::Close,
        gtk4::ResponseType::Yes => MessageBoxResponse::Yes,
        gtk4::ResponseType::No => MessageBoxResponse::No,
        gtk4::ResponseType::Other(res) => match res {
            CUSTOM_CANCEL => MessageBoxResponse::Cancel,
            CUSTOM_NO => MessageBoxResponse::No,
            CUSTOM_OK => MessageBoxResponse::Ok,
            CUSTOM_RETRY => MessageBoxResponse::Retry,
            CUSTOM_YES => MessageBoxResponse::Yes,
            CUSTOM_CLOSE => MessageBoxResponse::Close,
            _ => MessageBoxResponse::Custom(res),
        },
        gtk4::ResponseType::DeleteEvent | gtk4::ResponseType::Reject => MessageBoxResponse::Cancel,
        _ => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("unrecognized response: {:?}", res),
            ));
        }
    };
    Ok(res)
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

    pub async fn show(self, parent: Option<&Window>) -> io::Result<MessageBoxResponse> {
        msgbox_custom(
            parent, self.msg, self.title, self.instr, self.style, self.btns, self.cbtns,
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

    pub fn custom_button(mut self, btn: CustomButton) -> Self {
        self.cbtns.push(btn);
        self
    }

    pub fn custom_buttons(mut self, btn: impl IntoIterator<Item = CustomButton>) -> Self {
        self.cbtns.extend(btn);
        self
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct CustomButton {
    pub result: u16,
    pub text: String,
}

impl CustomButton {
    pub fn new(result: u16, text: impl AsRef<str>) -> Self {
        Self {
            result,
            text: text.as_ref().to_string(),
        }
    }
}
