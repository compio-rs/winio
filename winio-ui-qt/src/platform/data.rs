use std::{cell::OnceCell, ffi::OsString, fmt::Debug};

use cxx::UniquePtr;
pub(crate) use ffi::QMimeData;
use mime::Mime;

use crate::{Result, platform::Image};

pub struct Data {
    inner: UniquePtr<QMimeData>,
    entries: Vec<Entry>,
}

impl Debug for Data {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Data")
            .field("formats", &self.formats())
            .finish_non_exhaustive()
    }
}

#[derive(Debug)]
struct Entry {
    mime: Mime,
    bytes: OnceCell<Vec<u8>>,
}

impl Data {
    pub(crate) fn from_mime_data(data: &QMimeData) -> Result<Self> {
        let inner = ffi::clone_mime_data(data)?;
        Self::from_owned(inner)
    }

    pub(crate) fn from_owned(inner: UniquePtr<QMimeData>) -> Result<Self> {
        let has_image = ffi::mime_data_has_image(&inner);
        let mut entries = Vec::new();
        for format in ffi::mime_data_formats(&inner)? {
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
        if has_image && !entries.iter().any(|e| e.mime.type_() == mime::IMAGE) {
            entries.insert(0, new_entry(mime::IMAGE_PNG.clone()));
        }
        Ok(Self { inner, entries })
    }

    pub(crate) fn qmime_data(&self) -> &QMimeData {
        &self.inner
    }

    pub fn formats(&self) -> Vec<Mime> {
        self.entries.iter().map(|e| e.mime.clone()).collect()
    }

    pub fn get(&self, mime: &Mime) -> Option<&[u8]> {
        let entry = self.entries.iter().find(|e| &e.mime == mime)?;
        Some(entry.bytes.get_or_init(|| self.load_bytes(mime)).as_slice())
    }

    fn load_bytes(&self, mime: &Mime) -> Vec<u8> {
        let mut bytes = match ffi::mime_data_get(&self.inner, mime.as_ref()) {
            Ok(bytes) => bytes,
            Err(e) => {
                compio_log::error!("Failed to read clipboard data: {e:?}");
                return Vec::new();
            }
        };
        if bytes.is_empty()
            && mime.type_() == mime::IMAGE
            && mime.subtype() == mime::PNG
            && ffi::mime_data_has_image(&self.inner)
        {
            match ffi::mime_data_image_png(&self.inner) {
                Ok(png) => bytes = png,
                Err(e) => compio_log::error!("Failed to encode clipboard image: {e:?}"),
            }
        }
        bytes
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
            Ok(_) => Err(self),
            Err(e) => {
                compio_log::error!("Failed to read clipboard URIs: {e:?}");
                Err(self)
            }
        }
    }

    pub fn into_image(self) -> std::result::Result<Image, Self> {
        match ffi::mime_data_image(&self.inner) {
            Ok(image) if !image.is_null() => Ok(Image::from_qimage(image)),
            Ok(_) => Err(self),
            Err(e) => {
                compio_log::error!("Failed to read clipboard image: {e:?}");
                Err(self)
            }
        }
    }
}

fn new_entry(mime: Mime) -> Entry {
    Entry {
        mime,
        bytes: OnceCell::new(),
    }
}

fn push_unique(entries: &mut Vec<Entry>, mime: Mime) {
    if !entries.iter().any(|e| e.mime == mime) {
        entries.push(new_entry(mime));
    }
}

#[cxx::bridge]
mod ffi {
    unsafe extern "C++-unwind" {
        include!("winio-ui-qt/src/platform/data.hpp");

        type QImage = crate::platform::QImage;
        type QString = crate::common::QString;
        type QMimeData;

        fn clone_mime_data(data: &QMimeData) -> Result<UniquePtr<QMimeData>>;
        fn mime_data_formats(data: &QMimeData) -> Result<Vec<String>>;
        fn mime_data_get(data: &QMimeData, mime: &str) -> Result<Vec<u8>>;
        fn mime_data_text(data: &QMimeData, mime: &str) -> Result<QString>;
        fn mime_data_has_image(data: &QMimeData) -> bool;
        fn mime_data_image_png(data: &QMimeData) -> Result<Vec<u8>>;
        fn mime_data_image(data: &QMimeData) -> Result<UniquePtr<QImage>>;
        fn mime_data_local_files(data: &QMimeData) -> Result<Vec<String>>;
    }
}
