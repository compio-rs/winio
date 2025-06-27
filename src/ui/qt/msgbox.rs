use std::{collections::HashMap, mem::ManuallyDrop, ptr::null_mut};

use cxx::{ExternType, type_id};
use local_sync::oneshot;

use crate::{AsRawWindow, AsWindow, MessageBoxButton, MessageBoxResponse, MessageBoxStyle};

fn msgbox_finished(data: *const u8, res: i32) {
    if let Some(tx) = unsafe { (data.cast_mut() as *mut Option<oneshot::Sender<i32>>).as_mut() } {
        if let Some(tx) = tx.take() {
            tx.send(res).ok();
        }
    }
}

async fn msgbox_custom(
    parent: Option<impl AsWindow>,
    msg: String,
    title: String,
    instr: String,
    style: MessageBoxStyle,
    btns: MessageBoxButton,
    cbtns: Vec<CustomButton>,
) -> MessageBoxResponse {
    let parent = parent
        .map(|p| p.as_window().as_raw_window())
        .unwrap_or(null_mut());
    let mut b = unsafe { ffi::new_message_box(parent) };

    let mut results = HashMap::<usize, MessageBoxResponse>::new();
    if btns.contains(MessageBoxButton::Ok) {
        let key = b.pin_mut().addButton(QMessageBoxStandardButton::Ok) as usize;
        results.insert(key, MessageBoxResponse::Ok);
    }
    if btns.contains(MessageBoxButton::Yes) {
        let key = b.pin_mut().addButton(QMessageBoxStandardButton::Yes) as usize;
        results.insert(key, MessageBoxResponse::Yes);
    }
    if btns.contains(MessageBoxButton::No) {
        let key = b.pin_mut().addButton(QMessageBoxStandardButton::No) as usize;
        results.insert(key, MessageBoxResponse::No);
    }
    if btns.contains(MessageBoxButton::Cancel) {
        let key = b.pin_mut().addButton(QMessageBoxStandardButton::Cancel) as usize;
        results.insert(key, MessageBoxResponse::Cancel);
    }
    if btns.contains(MessageBoxButton::Retry) {
        let key = b.pin_mut().addButton(QMessageBoxStandardButton::Retry) as usize;
        results.insert(key, MessageBoxResponse::Retry);
    }
    if btns.contains(MessageBoxButton::Close) {
        let key = b.pin_mut().addButton(QMessageBoxStandardButton::Close) as usize;
        results.insert(key, MessageBoxResponse::Close);
    }
    for btn in &cbtns {
        let key = ffi::message_box_add_button(b.pin_mut(), &btn.text) as usize;
        results.insert(key, MessageBoxResponse::Custom(btn.result));
    }

    b.pin_mut().setWindowTitle(&title.into());
    if instr.is_empty() {
        b.pin_mut().setText(&msg.into());
    } else {
        b.pin_mut().setText(&instr.into());
        b.pin_mut().setInformativeText(&msg.into());
    }

    let icon = match style {
        MessageBoxStyle::None => QMessageBoxIcon::NoIcon,
        MessageBoxStyle::Info => QMessageBoxIcon::Information,
        MessageBoxStyle::Warning => QMessageBoxIcon::Warning,
        MessageBoxStyle::Error => QMessageBoxIcon::Critical,
    };
    b.pin_mut().setIcon(icon);

    let (tx, rx) = oneshot::channel::<i32>();
    let tx = ManuallyDrop::new(Some(tx));
    unsafe {
        ffi::message_box_connect_finished(
            b.pin_mut(),
            msgbox_finished,
            std::ptr::addr_of!(tx).cast(),
        );
    }
    b.pin_mut().open();
    rx.await.unwrap();

    let key = b.clickedButton() as usize;
    results[&key]
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

#[repr(i32)]
enum QMessageBoxIcon {
    NoIcon      = 0,
    Information = 2,
    Warning     = 3,
    Critical    = 4,
}

unsafe impl ExternType for QMessageBoxIcon {
    type Id = type_id!("QMessageBoxIcon");
    type Kind = cxx::kind::Trivial;
}

#[repr(i32)]
enum QMessageBoxStandardButton {
    Ok     = 0x00000400,
    Yes    = 0x00004000,
    No     = 0x00010000,
    Cancel = 0x00400000,
    Retry  = 0x00080000,
    Close  = 0x00200000,
}

unsafe impl ExternType for QMessageBoxStandardButton {
    type Id = type_id!("QMessageBoxStandardButton");
    type Kind = cxx::kind::Trivial;
}

#[cxx::bridge]
mod ffi {
    unsafe extern "C++-unwind" {
        include!("winio/src/ui/qt/msgbox.hpp");

        type QMessageBox;
        type QMessageBoxIcon = super::QMessageBoxIcon;
        type QMessageBoxStandardButton = super::QMessageBoxStandardButton;
        type QWidget = crate::ui::QWidget;
        type QPushButton;
        type QAbstractButton;
        type QString = crate::ui::QString;

        fn open(self: Pin<&mut QMessageBox>);
        fn setIcon(self: Pin<&mut QMessageBox>, icon: QMessageBoxIcon);
        fn setWindowTitle(self: Pin<&mut QMessageBox>, s: &QString);
        fn setText(self: Pin<&mut QMessageBox>, s: &QString);
        fn setInformativeText(self: Pin<&mut QMessageBox>, s: &QString);
        fn addButton(
            self: Pin<&mut QMessageBox>,
            button: QMessageBoxStandardButton,
        ) -> *mut QPushButton;
        fn clickedButton(self: &QMessageBox) -> *mut QAbstractButton;

        unsafe fn new_message_box(parent: *mut QWidget) -> UniquePtr<QMessageBox>;
        unsafe fn message_box_connect_finished(
            b: Pin<&mut QMessageBox>,
            callback: unsafe fn(*const u8, i32),
            data: *const u8,
        );
        fn message_box_add_button(b: Pin<&mut QMessageBox>, text: &str) -> *mut QPushButton;
    }
}
