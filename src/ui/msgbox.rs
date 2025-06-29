use crate::{MaybeBorrowedWindow, ui::sys};

/// Style of message box.
#[derive(Debug, Default, PartialEq, Eq, Clone, Copy)]
pub enum MessageBoxStyle {
    /// Simple message box.
    #[default]
    None,
    /// Show information.
    Info,
    /// Show warning message.
    Warning,
    /// Show error message.
    Error,
}

bitflags::bitflags! {
    /// The pre-defined message box buttons.
    #[derive(Debug, Default, PartialEq, Eq, Clone, Copy)]
    pub struct MessageBoxButton: i32 {
        /// "Ok"
        const Ok     = 1 << 0;
        /// "Yes"
        const Yes    = 1 << 1;
        /// "No"
        const No     = 1 << 2;
        /// "Cancel"
        const Cancel = 1 << 3;
        /// "Retry"
        const Retry  = 1 << 4;
        /// "Close"
        const Close  = 1 << 5;
    }
}

/// Response of message box.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum MessageBoxResponse {
    /// "Cancel"
    Cancel,
    /// "No"
    No,
    /// "Ok"
    Ok,
    /// "Retry"
    Retry,
    /// "Yes"
    Yes,
    /// "Close"
    Close,
    /// Custom response.
    Custom(u16),
}

/// Message box.
#[derive(Debug, Default, Clone)]
pub struct MessageBox(sys::MessageBox);

impl MessageBox {
    /// Create [`MessageBox`].
    pub fn new() -> Self {
        Self(sys::MessageBox::new())
    }

    /// Show message box.
    pub async fn show(self, parent: impl Into<MaybeBorrowedWindow<'_>>) -> MessageBoxResponse {
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
