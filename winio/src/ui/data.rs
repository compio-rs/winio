use std::ffi::OsString;

use inherit_methods_macro::inherit_methods;
use mime::Mime;

use crate::{sys, ui::Image};

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
    pub(crate) fn new(inner: sys::Data) -> Self {
        Self { inner }
    }

    pub(crate) fn sys(&self) -> &sys::Data {
        &self.inner
    }

    /// The available MIME formats, in the order of preference.
    pub fn formats(&self) -> Vec<Mime>;

    /// Get the raw bytes of the format `mime`.
    ///
    /// Returns `None` if the format is not available. For images, the bytes may
    /// be encoded on demand.
    pub fn get(&self, mime: &Mime) -> Option<&[u8]>;
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
    /// Convert the best available representation to [`KnownData`].
    ///
    /// The candidates are sorted by the priority image, URIs, HTML, RTF, plain
    /// text. If a representation cannot be converted, the next one is tried.
    /// If none can be converted, the original data is returned.
    pub fn into_known(self) -> Result<KnownData, Self> {
        let mut candidates = self
            .formats()
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
    fn into_known(self, data: Data) -> Result<KnownData, Data> {
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
