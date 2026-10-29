use std::ffi::OsString;

use inherit_methods_macro::inherit_methods;
use mime::Mime;

use crate::{Result, sys, ui::Image};

/// Data transferred through the clipboard or drag and drop.
///
/// The data may be available in multiple MIME formats. Use [`Data::formats`]
/// to list them and [`Data::get`] to read the raw bytes of a format.
#[derive(Debug)]
pub struct Data {
    inner: sys::Data,
}

#[inherit_methods(from = "self.inner")]
impl Data {
    pub(crate) fn from_sys(inner: sys::Data) -> Self {
        Self { inner }
    }

    pub(crate) fn sys(&self) -> &sys::Data {
        &self.inner
    }

    /// Insert a raw representation, replacing any existing data for `mime`.
    pub fn insert(&mut self, mime: Mime, bytes: impl AsRef<[u8]>) -> Result<()>;

    /// The available MIME formats, in the order of preference.
    ///
    /// Returns an error if the backend cannot enumerate the formats.
    pub fn formats(&self) -> Result<Vec<Mime>>;

    /// Get the raw bytes of the format `mime`.
    ///
    /// Returns `Ok(None)` if the format is not available. For images, the bytes
    /// may be encoded on demand.
    pub fn get(&self, mime: &Mime) -> Result<Option<&[u8]>>;
}

/// A typed representation of [`Data`].
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum KnownData {
    /// Plain text.
    Text(String),
    /// Rich Text Format.
    Rtf(String),
    /// HTML.
    Html(String),
    /// A bitmap image.
    Image(Image),
    /// A list of local files or URIs.
    Uris(Vec<OsString>),
}

impl Data {
    /// Create data with a single raw representation.
    pub fn new(mime: Mime, bytes: impl AsRef<[u8]>) -> Result<Self> {
        let mut data = Self::from_sys(sys::Data::new()?);
        data.insert(mime, bytes)?;
        Ok(data)
    }

    /// Create data from a typed representation.
    pub fn from_known(known: KnownData) -> Result<Self> {
        let mut data = Self::from_sys(sys::Data::new()?);
        data.insert_known(known)?;
        Ok(data)
    }

    /// Insert a typed representation, replacing formats as appropriate.
    pub fn insert_known(&mut self, known: KnownData) -> Result<()> {
        match known {
            KnownData::Text(text) => self.insert(mime::TEXT_PLAIN, text.as_bytes()),
            KnownData::Html(html) => self.insert(mime::TEXT_HTML, html.as_bytes()),
            KnownData::Rtf(rtf) => {
                let bytes = rtf.as_bytes();
                self.insert(parse_mime("text/rtf"), bytes)?;
                self.insert(parse_mime("application/rtf"), bytes)
            }
            KnownData::Image(image) => self.inner.insert_image(&image.0),
            KnownData::Uris(uris) => self.inner.insert_uris(&uris),
        }
    }

    /// Convert the best available representation to [`KnownData`].
    ///
    /// The candidates are sorted by the priority image, URIs, HTML, RTF, plain
    /// text. If a representation cannot be converted, the next one is tried.
    /// If none can be converted, or the formats cannot be read, the original
    /// data is returned.
    pub fn into_known(self) -> std::result::Result<KnownData, Self> {
        let Ok(formats) = self.formats() else {
            return Err(self);
        };
        let mut candidates = formats
            .into_iter()
            .filter_map(Candidate::new)
            .collect::<Vec<_>>();
        candidates.sort_by_key(|candidate| candidate.kind);
        candidates.dedup_by_key(|candidate| candidate.kind);

        let mut data = self;
        for candidate in candidates {
            match candidate.into_known(data) {
                Ok(known) => return Ok(known),
                Err(rest) => data = rest,
            }
        }

        Err(data)
    }
}

fn parse_mime(value: &str) -> Mime {
    value.parse().expect("valid MIME")
}

/// Priority of a [`KnownData`] representation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Kind {
    Image,
    Uris,
    Html,
    Rtf,
    Text,
}

struct Candidate {
    kind: Kind,
    mime: Mime,
}

impl Candidate {
    fn into_known(self, data: Data) -> std::result::Result<KnownData, Data> {
        let inner = data.inner;
        let result = match self.kind {
            Kind::Image => inner
                .into_image()
                .map(|image| KnownData::Image(Image(image))),
            Kind::Uris => inner.into_uris().map(KnownData::Uris),
            Kind::Html => inner.into_text(&self.mime).map(KnownData::Html),
            Kind::Rtf => inner.into_text(&self.mime).map(KnownData::Rtf),
            Kind::Text => inner.into_text(&self.mime).map(KnownData::Text),
        };
        result.map_err(|inner| Data { inner })
    }

    fn new(mime: Mime) -> Option<Self> {
        let kind = if mime.type_() == mime::IMAGE {
            Kind::Image
        } else if mime.type_() == mime::TEXT && mime.subtype() == "uri-list" {
            Kind::Uris
        } else if mime.type_() == mime::TEXT && mime.subtype() == mime::HTML {
            Kind::Html
        } else if mime.subtype() == "rtf" {
            Kind::Rtf
        } else if mime.type_() == mime::TEXT {
            Kind::Text
        } else {
            return None;
        };
        Some(Self { kind, mime })
    }
}
