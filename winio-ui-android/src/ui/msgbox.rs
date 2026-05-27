use winio_handle::MaybeBorrowedWindow;
use winio_primitive::{MessageBoxButton, MessageBoxResponse, MessageBoxStyle};

use crate::Result;

#[derive(Debug, Default, Clone)]
pub struct MessageBox;

impl MessageBox {
    pub fn new() -> Self {
        todo!()
    }

    pub fn message(&mut self, _msg: impl AsRef<str>) {
        todo!()
    }

    pub fn title(&mut self, _title: impl AsRef<str>) {
        todo!()
    }

    pub fn instruction(&mut self, _instr: impl AsRef<str>) {
        todo!()
    }

    pub fn style(&mut self, _style: MessageBoxStyle) {
        todo!()
    }

    pub fn buttons(&mut self, _buttons: MessageBoxButton) {
        todo!()
    }

    pub fn custom_button(&mut self, _btn: CustomButton) {
        todo!()
    }

    pub fn custom_buttons(&mut self, _btns: impl IntoIterator<Item = impl Into<CustomButton>>) {
        todo!()
    }

    pub async fn show(
        self,
        _parent: impl Into<MaybeBorrowedWindow<'_>>,
    ) -> Result<MessageBoxResponse> {
        todo!()
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct CustomButton;

impl CustomButton {
    pub fn new(_result: u16, _label: impl AsRef<str>) -> Self {
        todo!()
    }
}
