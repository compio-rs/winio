/// A portable keyboard key code.
///
/// Identifies a key for press and release events, including modifier keys.
/// Text input is delivered separately as Unicode characters.
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
    /// A single-byte key symbol in the active keyboard layout.
    ///
    /// Values use Latin-1, with ASCII letters represented by uppercase bytes
    /// (`b'A'` through `b'Z'`), independently of Shift and Caps Lock. This is a
    /// key identifier, not an input character.
    Char(u8),
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
    /// The Clear key.
    Clear,
    /// A Shift key, without distinguishing left and right.
    Shift,
    /// A Control key, without distinguishing left and right.
    Control,
    /// An Alt or Option key, without distinguishing left and right.
    Alt,
    /// The Alt Graph key used to select an alternate character group.
    AltGr,
    /// A Super, Windows, or Command key.
    Super,
    /// A Hyper key.
    Hyper,
    /// A Meta key.
    Meta,
}
