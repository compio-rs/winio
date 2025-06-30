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
