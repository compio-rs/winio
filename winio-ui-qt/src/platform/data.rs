use std::{ffi::OsString, fmt::Debug};

use cxx::UniquePtr;
pub(crate) use ffi::QMimeData;
use futures_util::Stream;
use itertools::{Either, Itertools};
use mime::Mime;
use winio_primitive::{DataEntry, DataFormat};

use crate::{Result, common::QString, platform::Image};

const UNKNOWN_FORMAT: &str = "application/vnd.winio.unknown";

pub struct Data {
    inner: UniquePtr<QMimeData>,
}

impl Debug for Data {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Data")
            .field("formats", &self.formats().map(|f| f.collect::<Vec<_>>()))
            .finish_non_exhaustive()
    }
}

impl Data {
    pub fn new() -> Result<Self> {
        Self::from_owned(ffi::new_mime_data())
    }

    pub(crate) fn from_mime_data(data: &QMimeData) -> Result<Self> {
        Self::from_owned(ffi::clone_mime_data(data)?)
    }

    pub(crate) fn from_owned(inner: UniquePtr<QMimeData>) -> Result<Self> {
        Ok(Self { inner })
    }

    pub(crate) fn qmime_data(&self) -> &QMimeData {
        &self.inner
    }

    pub async fn insert(&mut self, entry: DataEntry<Image>) -> Result<()> {
        match entry {
            DataEntry::Text(text) => {
                ffi::mime_data_set(self.inner.pin_mut(), "text/plain", text.as_bytes())?
            }
            DataEntry::Html(html) => {
                ffi::mime_data_set(self.inner.pin_mut(), "text/html", html.as_bytes())?
            }
            DataEntry::Image(image) => {
                ffi::mime_data_set_image(self.inner.pin_mut(), image.as_qimage())?
            }
            DataEntry::Path(path) => self.append_paths([path])?,
            DataEntry::Paths(paths) => self.append_paths(paths)?,
            DataEntry::Unknown(bytes) => {
                ffi::mime_data_set(self.inner.pin_mut(), UNKNOWN_FORMAT, &bytes)?
            }
        }
        Ok(())
    }

    pub fn is_empty(&self) -> bool {
        make_entries(&self.inner)
            .map(|entries| entries.is_empty())
            .unwrap_or(true)
    }

    pub fn formats(&self) -> Result<impl Iterator<Item = DataFormat>> {
        let formats = make_entries(&self.inner)?
            .into_iter()
            .map(|mime| format_of(&mime))
            .unique();
        Ok(formats)
    }

    fn append_paths(&mut self, additional: impl IntoIterator<Item = OsString>) -> Result<()> {
        let mut paths = ffi::mime_data_local_files(&self.inner).unwrap_or_default();
        paths.extend(
            additional
                .into_iter()
                .map(|path| path.to_string_lossy().into_owned()),
        );
        let paths = paths
            .iter()
            .map(|path| QString::try_from(path.as_str()))
            .collect::<std::result::Result<Vec<_>, _>>()?;
        ffi::mime_data_set_uris(self.inner.pin_mut(), &paths).map_err(crate::Error::from)
    }

    pub fn into_stream(self) -> impl Stream<Item = Result<DataEntry<Image>>> + Unpin {
        let Self { inner } = self;
        let entries = match make_entries(&inner) {
            Ok(entries) => Either::Left(entries.into_iter().map(move |mime| {
                match format_of(&mime) {
                    DataFormat::Text | DataFormat::Html => {
                        ffi::mime_data_text(&inner, mime.as_ref())
                            .map_err(crate::Error::from)
                            .and_then(|text| {
                                String::try_from(text).map_err(|_| crate::Error::NotSupported)
                            })
                            .map(|text| {
                                if mime.essence_str() == "text/html" {
                                    DataEntry::Html(text)
                                } else {
                                    DataEntry::Text(text)
                                }
                            })
                    }
                    DataFormat::Image => ffi::mime_data_image(&inner)
                        .map_err(crate::Error::from)
                        .and_then(|image| {
                            if image.is_null() {
                                Err(crate::Error::NotSupported)
                            } else {
                                Ok(DataEntry::Image(Image::from_qimage(image)))
                            }
                        }),
                    DataFormat::Path => ffi::mime_data_local_files(&inner)
                        .map_err(crate::Error::from)
                        .map(|files| {
                            DataEntry::Paths(files.into_iter().map(OsString::from).collect())
                        }),
                    DataFormat::Unknown => ffi::mime_data_get(&inner, mime.as_ref())
                        .map_err(crate::Error::from)
                        .map(|bytes| DataEntry::Unknown(bytes.to_vec())),
                }
            })),
            Err(error) => Either::Right(std::iter::once(Err(error))),
        };
        futures_util::stream::iter(entries)
    }
}

fn format_of(mime: &Mime) -> DataFormat {
    if mime.essence_str() == "text/plain" {
        DataFormat::Text
    } else if mime.essence_str() == "text/html" {
        DataFormat::Html
    } else if mime.essence_str() == "text/uri-list" {
        DataFormat::Path
    } else if mime.type_() == mime::IMAGE {
        DataFormat::Image
    } else {
        DataFormat::Unknown
    }
}

fn make_entries(inner: &QMimeData) -> Result<Vec<Mime>> {
    let has_image = ffi::mime_data_has_image(inner);
    let mut entries = Vec::new();
    for format in ffi::mime_data_formats(inner)? {
        let Ok(mime) = format.parse::<Mime>() else {
            continue;
        };
        if mime.essence_str() == "application/x-qt-image" {
            if has_image {
                push_unique(&mut entries, mime::IMAGE_PNG.clone());
            }
            continue;
        }
        push_unique(&mut entries, mime);
    }
    if has_image && !entries.iter().any(|mime| mime.type_() == mime::IMAGE) {
        entries.insert(0, mime::IMAGE_PNG.clone());
    }
    Ok(entries)
}

fn push_unique(entries: &mut Vec<Mime>, mime: Mime) {
    if !entries.iter().any(|entry| entry == &mime) {
        entries.push(mime);
    }
}

#[cxx::bridge]
mod ffi {
    unsafe extern "C++-unwind" {
        include!("winio-ui-qt/src/platform/data.hpp");

        type QByteArray = crate::common::QByteArray;
        type QImage = crate::platform::QImage;
        type QString = crate::common::QString;
        type QMimeData;

        fn new_mime_data() -> UniquePtr<QMimeData>;
        fn clone_mime_data(data: &QMimeData) -> Result<UniquePtr<QMimeData>>;
        fn mime_data_formats(data: &QMimeData) -> Result<Vec<String>>;
        fn mime_data_get(data: &QMimeData, mime: &str) -> Result<QByteArray>;
        fn mime_data_text(data: &QMimeData, mime: &str) -> Result<QString>;
        fn mime_data_set(data: Pin<&mut QMimeData>, mime: &str, bytes: &[u8]) -> Result<()>;
        fn mime_data_set_image(data: Pin<&mut QMimeData>, image: &QImage) -> Result<()>;
        fn mime_data_set_uris(data: Pin<&mut QMimeData>, uris: &[QString]) -> Result<()>;
        fn mime_data_has_image(data: &QMimeData) -> bool;
        fn mime_data_image(data: &QMimeData) -> Result<UniquePtr<QImage>>;
        fn mime_data_local_files(data: &QMimeData) -> Result<Vec<String>>;
    }
}
