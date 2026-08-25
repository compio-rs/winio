/// A portable keyboard key code.
///
/// Character keys follow the active keyboard layout. Text produced by an input
/// method can contain more than one character and should be handled separately.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
#[non_exhaustive]
pub enum KeyCode {
    /// A key that cannot be represented by a portable code.
    Unidentified,
    /// The Backspace key.
    Backspace,
    /// The Enter or Return key.
    Enter,
    /// The left arrow key.
    Left,
    /// The right arrow key.
    Right,
    /// The up arrow key.
    Up,
    /// The down arrow key.
    Down,
    /// The Home key.
    Home,
    /// The End key.
    End,
    /// The Page Up key.
    PageUp,
    /// The Page Down key.
    PageDown,
    /// The Tab key.
    Tab,
    /// The Delete key.
    Delete,
    /// The Insert key.
    Insert,
    /// A function key.
    F(u8),
    /// A character key interpreted using the active keyboard layout.
    Char(char),
    /// The Escape key.
    Esc,
    /// The Caps Lock key.
    CapsLock,
    /// The Scroll Lock key.
    ScrollLock,
    /// The Num Lock key.
    NumLock,
    /// The Print Screen key.
    PrintScreen,
    /// The Pause key.
    Pause,
    /// The context menu key.
    Menu,
}

bitflags::bitflags! {
    /// Modifier keys active during a keyboard event.
    #[derive(Debug, Default, PartialEq, Eq, Clone, Copy, Hash)]
    pub struct KeyModifiers: u8 {
        /// No modifier keys are active.
        const NONE    = 0;
        /// A Shift key is active.
        const SHIFT   = 1 << 0;
        /// A Control key is active.
        const CONTROL = 1 << 1;
        /// An Alt or Option key is active.
        const ALT     = 1 << 2;
        /// A Super, Windows, or Command key is active.
        const SUPER   = 1 << 3;
        /// A Hyper key is active.
        const HYPER   = 1 << 4;
        /// A Meta key is active.
        const META    = 1 << 5;
    }
}

bitflags::bitflags! {
    /// Additional state associated with a keyboard event.
    #[derive(Debug, Default, PartialEq, Eq, Clone, Copy, Hash)]
    pub struct KeyEventState: u8 {
        /// No additional state is active.
        const NONE      = 0;
        /// The event originated from the numeric keypad.
        const KEYPAD    = 1 << 0;
        /// Caps Lock is enabled for the event.
        const CAPS_LOCK = 1 << 1;
        /// Num Lock is enabled for the event.
        const NUM_LOCK  = 1 << 2;
    }
}

/// A keyboard input event with its active modifiers and keyboard state.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
#[non_exhaustive]
pub struct KeyEvent {
    /// The input key.
    pub code: KeyCode,
    /// Modifier keys active during the event.
    pub modifiers: KeyModifiers,
    /// Additional keyboard state for the event.
    pub state: KeyEventState,
}
