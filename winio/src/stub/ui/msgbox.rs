use winio_handle::MaybeBorrowedWindow;
use winio_primitive::{MessageBoxButton, MessageBoxResponse, MessageBoxStyle};

use crate::stub::{Result, not_impl};

#[derive(Debug, Default, Clone)]
pub struct MessageBox;

impl MessageBox {
    pub fn new() -> Self {
        not_impl()
    }

    pub async fn show(
        self,
        _parent: impl Into<MaybeBorrowedWindow<'_>>,
    ) -> Result<MessageBoxResponse> {
        not_impl()
    }

    pub fn message(&mut self, _msg: impl AsRef<str>) {
        not_impl()
    }

    pub fn title(&mut self, _title: impl AsRef<str>) {
        not_impl()
    }

    pub fn instruction(&mut self, _instr: impl AsRef<str>) {
        not_impl()
    }

    pub fn style(&mut self, _style: MessageBoxStyle) {
        not_impl()
    }

    pub fn buttons(&mut self, _btns: MessageBoxButton) {
        not_impl()
    }

    pub fn custom_button(&mut self, _btn: impl Into<CustomButton>) {
        not_impl()
    }

    pub fn custom_buttons(&mut self, _btns: impl IntoIterator<Item = impl Into<CustomButton>>) {
        not_impl()
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct CustomButton;

impl CustomButton {
    pub fn new(_result: u16, _label: impl AsRef<str>) -> Self {
        not_impl()
    }
}
