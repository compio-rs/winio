use std::{cell::OnceCell, ffi::OsString, fmt::Debug};

use cxx::UniquePtr;
pub(crate) use ffi::QMimeData;
use mime::Mime;

use crate::{
    Result,
    common::{QByteArray, QString},
    platform::Image,
};

pub struct Data {
    inner: UniquePtr<QMimeData>,
    entries: Vec<Entry>,
}

impl Debug for Data {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Data")
            .field(
                "formats",
                &self
                    .entries
                    .iter()
                    .map(|entry| &entry.mime)
                    .collect::<Vec<_>>(),
            )
            .finish_non_exhaustive()
    }
}

struct Entry {
    mime: Mime,
    bytes: OnceCell<QByteArray>,
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

    pub fn insert(&mut self, mime: Mime, bytes: impl AsRef<[u8]>) -> Result<()> {
        ffi::mime_data_set(self.inner.pin_mut(), mime.as_ref(), bytes.as_ref())?;
        let index = self.entries.iter().position(|entry| entry.mime == mime);
        match index {
            Some(index) => self.entries[index] = new_entry(mime),
            None => self.entries.push(new_entry(mime)),
        }
        Ok(())
    }

    pub fn insert_image(&mut self, image: &Image) -> Result<()> {
        ffi::mime_data_set_image(self.inner.pin_mut(), image.as_qimage())?;
        self.rebuild_entries()
    }

    pub fn insert_uris(&mut self, uris: &[OsString]) -> Result<()> {
        let uris = uris
            .iter()
            .map(|uri| QString::try_from(uri.to_string_lossy().as_ref()))
            .collect::<std::result::Result<Vec<_>, _>>()?;
        ffi::mime_data_set_uris(self.inner.pin_mut(), &uris)?;
        self.rebuild_entries()
    }

    pub fn formats(&self) -> Result<Vec<Mime>> {
        Ok(self
            .entries
            .iter()
            .map(|entry| entry.mime.clone())
            .collect())
    }

    pub fn get(&self, mime: &Mime) -> Result<Option<&[u8]>> {
        let Some(entry) = self.entries.iter().find(|entry| &entry.mime == mime) else {
            return Ok(None);
        };
        if entry.bytes.get().is_none() {
            let bytes = ffi::mime_data_get(&self.inner, mime.as_ref())?;
            if bytes.is_empty()
                && mime.type_() == mime::IMAGE
                && mime.subtype() == mime::PNG
                && ffi::mime_data_has_image(&self.inner)
            {
                return Ok(None);
            }
            let _ = entry.bytes.set(bytes);
        }
        Ok(entry.bytes.get().map(|bytes| &**bytes))
    }

    pub fn into_text(self, mime: &Mime) -> std::result::Result<String, Self> {
        let text = ffi::mime_data_text(&self.inner, mime.as_ref())
            .ok()
            .and_then(|text| String::try_from(text).ok());
        match text {
            Some(text) => Ok(text),
            None => Err(self),
        }
    }

    pub fn into_uris(self) -> std::result::Result<Vec<OsString>, Self> {
        match ffi::mime_data_local_files(&self.inner) {
            Ok(files) if !files.is_empty() => Ok(files.into_iter().map(OsString::from).collect()),
            _ => Err(self),
        }
    }

    pub fn into_image(self) -> std::result::Result<Image, Self> {
        match ffi::mime_data_image(&self.inner) {
            Ok(image) if !image.is_null() => Ok(Image::from_qimage(image)),
            _ => Err(self),
        }
    }

    fn rebuild_entries(&mut self) -> Result<()> {
        self.entries = make_entries(&self.inner)?;
        Ok(())
    }
}

fn make_entries(inner: &QMimeData) -> Result<Vec<Entry>> {
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
    if has_image
        && !entries
            .iter()
            .any(|entry| entry.mime.type_() == mime::IMAGE)
    {
        entries.insert(0, new_entry(mime::IMAGE_PNG.clone()));
    }
    Ok(entries)
}

fn new_entry(mime: Mime) -> Entry {
    Entry {
        mime,
        bytes: OnceCell::new(),
    }
}

fn push_unique(entries: &mut Vec<Entry>, mime: Mime) {
    if !entries.iter().any(|entry| entry.mime == mime) {
        entries.push(new_entry(mime));
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
