use std::ffi::OsString;

/// The kind of data stored in a [`DataEntry`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DataFormat {
    /// Plain text.
    Text,
    /// HTML.
    Html,
    /// A bitmap image.
    Image,
    /// A local filesystem path.
    Path,
    /// An opaque byte payload.
    Unknown,
}

/// A single entry of clipboard or drag-and-drop data.
#[derive(Debug, Clone)]
pub enum DataEntry<I> {
    /// Plain text.
    Text(String),
    /// HTML.
    Html(String),
    /// A bitmap image.
    Image(I),
    /// A local filesystem path.
    Path(OsString),
    /// Multiple local filesystem paths.
    Paths(Vec<OsString>),
    /// An opaque byte payload.
    Unknown(Vec<u8>),
}

impl<I> DataEntry<I> {
    /// The format of this entry.
    pub fn format(&self) -> DataFormat {
        match self {
            Self::Text(_) => DataFormat::Text,
            Self::Html(_) => DataFormat::Html,
            Self::Image(_) => DataFormat::Image,
            Self::Path(_) | Self::Paths(_) => DataFormat::Path,
            Self::Unknown(_) => DataFormat::Unknown,
        }
    }

    /// Map the image payload of this entry, allowing the mapping to fail.
    pub fn try_map_image<J, E>(
        self,
        f: impl FnOnce(I) -> std::result::Result<J, E>,
    ) -> std::result::Result<DataEntry<J>, E> {
        match self {
            Self::Text(text) => Ok(DataEntry::Text(text)),
            Self::Html(html) => Ok(DataEntry::Html(html)),
            Self::Image(image) => f(image).map(DataEntry::Image),
            Self::Path(path) => Ok(DataEntry::Path(path)),
            Self::Paths(paths) => Ok(DataEntry::Paths(paths)),
            Self::Unknown(bytes) => Ok(DataEntry::Unknown(bytes)),
        }
    }

    /// Map the image payload of this entry.
    pub fn map_image<J>(self, f: impl FnOnce(I) -> J) -> DataEntry<J> {
        match self {
            Self::Text(text) => DataEntry::Text(text),
            Self::Html(html) => DataEntry::Html(html),
            Self::Image(image) => DataEntry::Image(f(image)),
            Self::Path(path) => DataEntry::Path(path),
            Self::Paths(paths) => DataEntry::Paths(paths),
            Self::Unknown(bytes) => DataEntry::Unknown(bytes),
        }
    }
}
