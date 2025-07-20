use {
    winio_handle::AsWindow,
    winio_primitive::{MessageBoxButton, MessageBoxResponse, MessageBoxStyle},
};

#[derive(Debug, Default, Clone)]
pub struct MessageBox;

impl MessageBox {
    pub fn new() -> Self {
        todo!()
    }

    pub fn message<S>(&mut self, _msg: S)
    where
        S: AsRef<str>,
    {
        todo!()
    }

    pub fn title<S>(&mut self, _title: S)
    where
        S: AsRef<str>,
    {
        todo!()
    }

    pub fn instruction<S>(&mut self, _instr: S)
    where
        S: AsRef<str>,
    {
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

    pub fn custom_buttons<I>(&mut self, _buttons: I)
    where
        I: IntoIterator<Item = CustomButton>,
    {
        todo!()
    }

    pub async fn show<W>(self, _parent: Option<W>) -> MessageBoxResponse
    where
        W: AsWindow,
    {
        todo!()
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct CustomButton;

impl CustomButton {
    pub fn new<S>(_result: u16, _text: S) -> Self
    where
        S: AsRef<str>,
    {
        todo!()
    }
}
