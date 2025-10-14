#![warn(missing_docs)]

/// Backdrop effects for windows.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
#[non_exhaustive]
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
