use widestring::U16CString;
use winio_handle::AsWindow;
use winio_primitive::{MessageBoxButton, MessageBoxResponse, MessageBoxStyle};
pub use winio_ui_windows_common::CustomButton;
use winio_ui_windows_common::msgbox;

use crate::darkmode::TASK_DIALOG_CALLBACK;

#[derive(Debug, Clone)]
pub struct MessageBox {
    msg: U16CString,
    title: U16CString,
    instr: U16CString,
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
            msg: U16CString::new(),
            title: U16CString::new(),
            instr: U16CString::new(),
            style: MessageBoxStyle::None,
            btns: MessageBoxButton::empty(),
            cbtns: vec![],
        }
    }

    pub async fn show(self, parent: Option<impl AsWindow>) -> MessageBoxResponse {
        let parent = parent.map(|parent| parent.as_window().as_win32());
        msgbox(
            parent,
            self.msg,
            self.title,
            self.instr,
            self.style,
            self.btns,
            self.cbtns,
            TASK_DIALOG_CALLBACK,
        )
        .await
    }

    pub fn message(&mut self, msg: &str) {
        self.msg = U16CString::from_str_truncate(msg);
    }

    pub fn title(&mut self, title: &str) {
        self.title = U16CString::from_str_truncate(title);
    }

    pub fn instruction(&mut self, instr: &str) {
        self.instr = U16CString::from_str_truncate(instr);
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
