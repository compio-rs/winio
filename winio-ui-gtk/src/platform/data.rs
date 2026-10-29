use std::{
    ffi::OsString,
    path::{Path, PathBuf},
};

use gtk4::{
    gdk, gio,
    glib::{self, Bytes},
    prelude::*,
};
use mime::Mime;

use crate::{Result, platform::Image};

#[derive(Debug)]
pub struct Data {
    formats: Vec<Mime>,
    entries: Vec<(Mime, Vec<u8>)>,
    image: Option<Image>,
    uris: Option<Vec<OsString>>,
}

impl Data {
    pub fn new() -> Result<Self> {
        Ok(Self {
            formats: Vec::new(),
            entries: Vec::new(),
            image: None,
            uris: None,
        })
    }

    pub(crate) async fn from_clipboard(clipboard: &gdk::Clipboard) -> Result<Option<Self>> {
        let formats = clipboard.formats();
        let mut data = Self::new()?;

        let has_image = formats.contains_type(gdk::Texture::static_type())
            || formats
                .mime_types()
                .iter()
                .any(|mime| mime.starts_with("image/"));
        if has_image
            && let Ok(value) = clipboard
                .read_value_future(gdk::Texture::static_type(), glib::Priority::DEFAULT)
                .await
            && let Ok(texture) = value.get::<gdk::Texture>()
        {
            data.insert_image(&Image::from_texture(texture))?;
        }

        if formats.contains_type(gdk::FileList::static_type())
            && let Ok(value) = clipboard
                .read_value_future(gdk::FileList::static_type(), glib::Priority::DEFAULT)
                .await
            && let Ok(files) = value.get::<gdk::FileList>()
        {
            data.insert_files(files.files())?;
        }

        for mime in formats.mime_types() {
            let Ok(typed) = mime.as_str().parse::<Mime>() else {
                continue;
            };
            if data.image.is_some() && typed.type_() == mime::IMAGE {
                data.insert_format(typed);
                continue;
            }
            if data.uris.is_some() && typed.essence_str() == "text/uri-list" {
                data.insert_format(typed);
                continue;
            }
            if let Ok((stream, actual)) = clipboard
                .read_future(&[mime.as_str()], glib::Priority::DEFAULT)
                .await
                && let Ok(bytes) = read_stream(stream).await
                && let Ok(actual) = actual.as_str().parse::<Mime>()
            {
                data.insert(actual, bytes)?;
            }
        }

        if data.is_empty() {
            Ok(None)
        } else {
            Ok(Some(data))
        }
    }

    pub(crate) async fn from_drop(drop: &gdk::Drop) -> Result<Option<Self>> {
        let formats = drop.formats();
        let mut data = Self::new()?;

        let has_image = formats.contains_type(gdk::Texture::static_type())
            || formats
                .mime_types()
                .iter()
                .any(|mime| mime.starts_with("image/"));
        if has_image
            && let Ok(value) = drop
                .read_value_future(gdk::Texture::static_type(), glib::Priority::DEFAULT)
                .await
            && let Ok(texture) = value.get::<gdk::Texture>()
        {
            data.insert_image(&Image::from_texture(texture))?;
        }

        if formats.contains_type(gdk::FileList::static_type())
            && let Ok(value) = drop
                .read_value_future(gdk::FileList::static_type(), glib::Priority::DEFAULT)
                .await
            && let Ok(files) = value.get::<gdk::FileList>()
        {
            data.insert_files(files.files())?;
        }

        for mime in formats.mime_types() {
            let Ok(typed) = mime.as_str().parse::<Mime>() else {
                continue;
            };
            if data.image.is_some() && typed.type_() == mime::IMAGE {
                data.insert_format(typed);
                continue;
            }
            if data.uris.is_some() && typed.essence_str() == "text/uri-list" {
                data.insert_format(typed);
                continue;
            }
            if let Ok((stream, actual)) = drop
                .read_future(&[mime.as_str()], glib::Priority::DEFAULT)
                .await
                && let Ok(bytes) = read_stream(stream).await
                && let Ok(actual) = actual.as_str().parse::<Mime>()
            {
                data.insert(actual, bytes)?;
            }
        }

        if data.is_empty() {
            Ok(None)
        } else {
            Ok(Some(data))
        }
    }

    fn insert_format(&mut self, mime: Mime) {
        if !self.formats.iter().any(|entry| entry == &mime) {
            self.formats.push(mime);
        }
    }

    pub fn insert(&mut self, mime: Mime, bytes: impl AsRef<[u8]>) -> Result<()> {
        let bytes = bytes.as_ref().to_vec();
        if let Some((_, value)) = self.entries.iter_mut().find(|(entry, _)| entry == &mime) {
            *value = bytes;
        } else {
            self.formats.push(mime.clone());
            self.entries.push((mime, bytes));
        }
        Ok(())
    }

    pub fn insert_image(&mut self, image: &Image) -> Result<()> {
        self.image = Some(image.clone());
        if !self.formats.iter().any(|mime| mime.type_() == mime::IMAGE) {
            self.formats.push(mime::IMAGE_PNG.clone());
        }
        Ok(())
    }

    pub fn insert_uris(&mut self, uris: &[OsString]) -> Result<()> {
        let bytes = encode_uris(uris)?;
        self.uris = Some(uris.to_vec());
        self.insert(parse_mime("text/uri-list"), bytes)?;
        Ok(())
    }

    fn insert_files(&mut self, files: Vec<gio::File>) -> Result<()> {
        let uris = files
            .into_iter()
            .filter_map(|file| file.path())
            .map(PathBuf::into_os_string)
            .collect::<Vec<_>>();
        if uris.is_empty() {
            Ok(())
        } else {
            self.insert_uris(&uris)
        }
    }

    fn is_empty(&self) -> bool {
        self.entries.is_empty() && self.image.is_none() && self.uris.is_none()
    }

    pub fn formats(&self) -> Result<Vec<Mime>> {
        Ok(self.formats.clone())
    }

    pub fn get(&self, mime: &Mime) -> Result<Option<&[u8]>> {
        Ok(self
            .entries
            .iter()
            .find(|(entry, _)| entry == mime)
            .map(|(_, bytes)| bytes.as_slice()))
    }

    pub fn into_text(self, mime: &Mime) -> std::result::Result<String, Self> {
        let text = self
            .entries
            .iter()
            .find(|(entry, _)| entry == mime)
            .and_then(|(_, bytes)| String::from_utf8(bytes.clone()).ok());
        match text {
            Some(text) => Ok(text),
            None => Err(self),
        }
    }

    pub fn into_uris(self) -> std::result::Result<Vec<OsString>, Self> {
        if let Some(uris) = self.uris {
            return Ok(uris);
        }
        let Some((_, bytes)) = self
            .entries
            .iter()
            .find(|(entry, _)| entry.essence_str() == "text/uri-list")
        else {
            return Err(self);
        };
        let Ok(text) = std::str::from_utf8(bytes) else {
            return Err(self);
        };
        let uris = text
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty() && !line.starts_with('#'))
            .filter_map(|uri| gio::File::for_uri(uri).path())
            .map(PathBuf::into_os_string)
            .collect::<Vec<_>>();
        if uris.is_empty() { Err(self) } else { Ok(uris) }
    }

    pub fn into_image(self) -> std::result::Result<Image, Self> {
        if let Some(image) = self.image {
            return Ok(image);
        }
        for (mime, bytes) in &self.entries {
            if mime.type_() == mime::IMAGE
                && let Ok(texture) = gdk::Texture::from_bytes(&Bytes::from_owned(bytes.clone()))
            {
                return Ok(Image::from_texture(texture));
            }
        }
        Err(self)
    }

    pub(crate) fn content_provider(&self) -> Result<Option<gdk::ContentProvider>> {
        let mut providers = Vec::new();
        for (mime, bytes) in &self.entries {
            providers.push(gdk::ContentProvider::for_bytes(
                mime.as_ref(),
                &Bytes::from_owned(bytes.clone()),
            ));
        }
        if let Some(image) = &self.image {
            providers.push(gdk::ContentProvider::for_value(&image.texture().to_value()));
        }
        if let Some(uris) = &self.uris {
            let files = uris
                .iter()
                .map(|uri| gio::File::for_path(Path::new(uri)))
                .collect::<Vec<_>>();
            if !files.is_empty() {
                let list = gdk::FileList::from_array(&files);
                providers.push(gdk::ContentProvider::for_value(&list.to_value()));
            }
        }
        match providers.len() {
            0 => Ok(None),
            1 => Ok(providers.pop()),
            _ => Ok(Some(gdk::ContentProvider::new_union(&providers))),
        }
    }
}

async fn read_stream(stream: gio::InputStream) -> Result<Vec<u8>> {
    let mut result = Vec::new();
    loop {
        let bytes = stream
            .read_bytes_future(8192, glib::Priority::DEFAULT)
            .await?;
        if bytes.is_empty() {
            break;
        }
        result.extend_from_slice(&bytes);
    }
    Ok(result)
}

fn parse_mime(value: &str) -> Mime {
    value.parse().expect("valid MIME")
}

fn encode_uris(uris: &[OsString]) -> Result<Vec<u8>> {
    let mut text = String::new();
    for uri in uris {
        text.push_str(gio::File::for_path(Path::new(uri)).uri().as_str());
        text.push_str("\r\n");
    }
    Ok(text.into_bytes())
}
