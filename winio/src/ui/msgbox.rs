use winio_handle::MaybeBorrowedWindow;
use winio_primitive::{MessageBoxButton, MessageBoxResponse, MessageBoxStyle};

use crate::{sys, sys::Result};

/// Message box.
#[derive(Debug, Default, Clone)]
pub struct MessageBox(sys::MessageBox);

impl MessageBox {
    /// Create [`MessageBox`].
    pub fn new() -> Self {
        Self(sys::MessageBox::new())
    }

    /// Show message box.
    pub async fn show(
        self,
        parent: impl Into<MaybeBorrowedWindow<'_>>,
    ) -> Result<MessageBoxResponse> {
        self.0.show(parent.into().0).await
    }

    /// Main message.
    pub fn message(mut self, msg: impl AsRef<str>) -> Self {
        self.0.message(msg.as_ref());
        self
    }

    /// Box title.
    pub fn title(mut self, title: impl AsRef<str>) -> Self {
        self.0.title(title.as_ref());
        self
    }

    /// Optional instruction title.
    pub fn instruction(mut self, instr: impl AsRef<str>) -> Self {
        self.0.instruction(instr.as_ref());
        self
    }

    /// Style.
    pub fn style(mut self, style: MessageBoxStyle) -> Self {
        self.0.style(style);
        self
    }

    /// Pre-defined buttons.
    pub fn buttons(mut self, btns: MessageBoxButton) -> Self {
        self.0.buttons(btns);
        self
    }

    /// Add a custom button.
    pub fn custom_button(mut self, btn: impl Into<CustomButton>) -> Self {
        self.0.custom_button(btn.into().0);
        self
    }

    /// Set custom buttons.
    pub fn custom_buttons(mut self, btn: impl IntoIterator<Item = CustomButton>) -> Self {
        self.0.custom_buttons(btn.into_iter().map(|b| b.0));
        self
    }
}

/// Custom button for [`MessageBox`].
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct CustomButton(sys::CustomButton);

impl CustomButton {
    /// Create [`CustomButton`].
    pub fn new(result: u16, text: impl AsRef<str>) -> Self {
        Self(sys::CustomButton::new(result, text.as_ref()))
    }
}

impl<S: AsRef<str>> From<(u16, S)> for CustomButton {
    fn from((result, text): (u16, S)) -> Self {
        Self::new(result, text)
    }
}
