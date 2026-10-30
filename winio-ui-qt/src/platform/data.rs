use std::{ffi::OsString, fmt::Debug};

use cxx::UniquePtr;
pub(crate) use ffi::QMimeData;
use mime::Mime;
use winio_primitive::{DataEntry, DataFormat};

use crate::{Result, common::QString, platform::Image};

const UNKNOWN_FORMAT: &str = "application/vnd.winio.unknown";

pub struct Data {
    inner: UniquePtr<QMimeData>,
    entries: Vec<Mime>,
}

impl Debug for Data {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Data")
            .field("formats", &self.entries)
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
        let entries = make_entries(&inner)?;
        Ok(Self { inner, entries })
    }

    pub(crate) fn qmime_data(&self) -> &QMimeData {
        &self.inner
    }

    pub fn insert_text(&mut self, text: String) -> Result<()> {
        ffi::mime_data_set(self.inner.pin_mut(), "text/plain", text.as_bytes())?;
        self.rebuild_entries()
    }

    pub fn insert_html(&mut self, html: String) -> Result<()> {
        ffi::mime_data_set(self.inner.pin_mut(), "text/html", html.as_bytes())?;
        self.rebuild_entries()
    }

    pub fn insert_image(&mut self, image: &Image) -> Result<()> {
        ffi::mime_data_set_image(self.inner.pin_mut(), image.as_qimage())?;
        self.rebuild_entries()
    }

    pub fn insert_paths(&mut self, paths: &[OsString]) -> Result<()> {
        let paths = paths
            .iter()
            .map(|path| QString::try_from(path.to_string_lossy().as_ref()))
            .collect::<std::result::Result<Vec<_>, _>>()?;
        ffi::mime_data_set_uris(self.inner.pin_mut(), &paths)?;
        self.rebuild_entries()
    }

    pub fn insert_unknown(&mut self, bytes: Vec<u8>) -> Result<()> {
        ffi::mime_data_set(self.inner.pin_mut(), UNKNOWN_FORMAT, &bytes)?;
        self.rebuild_entries()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn formats(&self) -> Result<impl Iterator<Item = DataFormat> + '_> {
        Ok(self.entries.iter().map(format_of))
    }

    fn rebuild_entries(&mut self) -> Result<()> {
        self.entries = make_entries(&self.inner)?;
        Ok(())
    }
}

impl IntoIterator for Data {
    type IntoIter = DataIntoIter;
    type Item = Result<DataEntry<Image>>;

    fn into_iter(self) -> Self::IntoIter {
        DataIntoIter(
            self.entries
                .into_iter()
                .map(|mime| match format_of(&mime) {
                    DataFormat::Text | DataFormat::Html => {
                        let text = ffi::mime_data_text(&self.inner, mime.as_ref())?;
                        let text =
                            String::try_from(text).map_err(|_| crate::Error::NotSupported)?;
                        Ok(if mime.essence_str() == "text/html" {
                            DataEntry::Html(text)
                        } else {
                            DataEntry::Text(text)
                        })
                    }
                    DataFormat::Image => match ffi::mime_data_image(&self.inner) {
                        Ok(image) if !image.is_null() => {
                            Ok(DataEntry::Image(Image::from_qimage(image)))
                        }
                        _ => Err(crate::Error::NotSupported),
                    },
                    DataFormat::Paths => Ok(DataEntry::Paths(
                        ffi::mime_data_local_files(&self.inner)?
                            .into_iter()
                            .map(OsString::from)
                            .collect(),
                    )),
                    _ => Ok(DataEntry::Unknown(
                        ffi::mime_data_get(&self.inner, mime.as_ref())?.to_vec(),
                    )),
                })
                .collect::<Vec<_>>()
                .into_iter(),
        )
    }
}

pub struct DataIntoIter(std::vec::IntoIter<Result<DataEntry<Image>>>);

impl Iterator for DataIntoIter {
    type Item = Result<DataEntry<Image>>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

fn format_of(mime: &Mime) -> DataFormat {
    if mime.essence_str() == "text/plain" {
        DataFormat::Text
    } else if mime.essence_str() == "text/html" {
        DataFormat::Html
    } else if mime.essence_str() == "text/uri-list" {
        DataFormat::Paths
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
