pub struct Window;

// TODO: do we have to write it here again?

/// Backdrop effects for windows.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
#[non_exhaustive]
#[cfg(windows)]
pub enum Backdrop {
    /// Default window style.
    None,
    /// Acrylic effect.
    Acrylic,
    /// Mica effect.
    Mica,
    /// Mica Alt effect.
    MicaAlt,
}
