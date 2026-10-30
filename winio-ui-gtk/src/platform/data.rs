use std::{
    ffi::OsString,
    fmt::Debug,
    future::Future,
    path::{Path, PathBuf},
    pin::Pin,
};

use gtk4::{
    gdk, gio,
    glib::{self, Bytes},
    prelude::*,
};
use mime::Mime;
use winio_primitive::{DataEntry, DataFormat};

use crate::{Error, Result, platform::Image};

pub struct Data {
    entries: Vec<Entry>,
}

impl Debug for Data {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Data")
            .field(
                "formats",
                &self.entries.iter().map(Entry::format).collect::<Vec<_>>(),
            )
            .finish_non_exhaustive()
    }
}

struct Entry {
    mime: Mime,
    value: Value,
}

enum Value {
    Bytes(Vec<u8>),
    Image(Image),
    Paths(Vec<OsString>),
}

impl Entry {
    fn format(&self) -> DataFormat {
        format_of(&self.mime)
    }

    fn into_entry(self) -> Result<DataEntry<Image>> {
        let format = self.format();
        match self.value {
            Value::Image(image) => Ok(DataEntry::Image(image)),
            Value::Paths(paths) => Ok(DataEntry::Paths(paths)),
            Value::Bytes(bytes) => match format {
                DataFormat::Text => String::from_utf8(bytes)
                    .map(DataEntry::Text)
                    .map_err(|_| Error::NotSupported),
                DataFormat::Html => String::from_utf8(bytes)
                    .map(DataEntry::Html)
                    .map_err(|_| Error::NotSupported),
                DataFormat::Paths => decode_uri_list(&bytes)
                    .map(DataEntry::Paths)
                    .ok_or(Error::NotSupported),
                DataFormat::Image => gdk::Texture::from_bytes(&Bytes::from_owned(bytes))
                    .map(|texture| DataEntry::Image(Image::from_texture(texture)))
                    .map_err(|_| Error::NotSupported),
                DataFormat::Unknown => Ok(DataEntry::Unknown(bytes)),
            },
        }
    }
}

impl Data {
    pub fn new() -> Result<Self> {
        Ok(Self {
            entries: Vec::new(),
        })
    }

    pub(crate) async fn from_clipboard(clipboard: &gdk::Clipboard) -> Result<Option<Self>> {
        Self::from_source(clipboard).await
    }

    pub(crate) async fn from_drop(drop: &gdk::Drop) -> Result<Option<Self>> {
        Self::from_source(drop).await
    }

    async fn from_source(source: &impl ContentSource) -> Result<Option<Self>> {
        let formats = source.formats();
        let mut data = Self::new()?;

        let has_image = formats.contains_type(gdk::Texture::static_type())
            || formats
                .mime_types()
                .iter()
                .any(|mime| mime.starts_with("image/"));
        let mut image = false;
        if has_image
            && let Ok(value) = source.read_value(gdk::Texture::static_type()).await
            && let Ok(texture) = value.get::<gdk::Texture>()
        {
            data.insert_image(&Image::from_texture(texture))?;
            image = true;
        }

        let mut paths = false;
        if formats.contains_type(gdk::FileList::static_type())
            && let Ok(value) = source.read_value(gdk::FileList::static_type()).await
            && let Ok(files) = value.get::<gdk::FileList>()
        {
            let uris = files
                .files()
                .into_iter()
                .filter_map(|file| file.path())
                .map(PathBuf::into_os_string)
                .collect::<Vec<_>>();
            if !uris.is_empty() {
                data.insert_paths(&uris)?;
                paths = true;
            }
        }

        for mime in formats.mime_types() {
            let Ok(typed) = mime.as_str().parse::<Mime>() else {
                continue;
            };
            if image && typed.type_() == mime::IMAGE {
                continue;
            }
            if paths && typed.essence_str() == "text/uri-list" {
                continue;
            }
            if let Ok((stream, actual)) = source.read_mime(mime.as_str()).await
                && let Ok(bytes) = read_stream(stream).await
                && let Ok(actual) = actual.as_str().parse::<Mime>()
            {
                data.insert_bytes(actual, bytes)?;
            }
        }

        if data.is_empty() {
            Ok(None)
        } else {
            Ok(Some(data))
        }
    }

    pub fn insert_text(&mut self, text: String) -> Result<()> {
        self.insert_bytes(mime::TEXT_PLAIN, text.into_bytes())
    }

    pub fn insert_html(&mut self, html: String) -> Result<()> {
        self.insert_bytes(mime::TEXT_HTML, html.into_bytes())
    }

    pub fn insert_image(&mut self, image: &Image) -> Result<()> {
        self.insert_entry(Entry {
            mime: mime::IMAGE_PNG.clone(),
            value: Value::Image(image.clone()),
        });
        Ok(())
    }

    pub fn insert_paths(&mut self, paths: &[OsString]) -> Result<()> {
        self.insert_entry(Entry {
            mime: parse_mime("text/uri-list"),
            value: Value::Paths(paths.to_vec()),
        });
        Ok(())
    }

    pub fn insert_unknown(&mut self, bytes: Vec<u8>) -> Result<()> {
        self.insert_bytes(parse_mime("application/vnd.winio.unknown"), bytes)
    }

    fn insert_bytes(&mut self, mime: Mime, bytes: Vec<u8>) -> Result<()> {
        self.insert_entry(Entry {
            mime,
            value: Value::Bytes(bytes),
        });
        Ok(())
    }

    fn insert_entry(&mut self, entry: Entry) {
        let format = entry.format();
        if format == DataFormat::Unknown {
            self.entries.retain(|existing| existing.mime != entry.mime);
        } else {
            self.entries.retain(|existing| existing.format() != format);
        }
        self.entries.push(entry);
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn formats(&self) -> Result<impl Iterator<Item = DataFormat> + '_> {
        Ok(self.entries.iter().map(Entry::format))
    }

    pub(crate) fn content_provider(&self) -> Result<Option<gdk::ContentProvider>> {
        let mut providers = Vec::new();
        for entry in &self.entries {
            match &entry.value {
                Value::Bytes(bytes) => providers.push(gdk::ContentProvider::for_bytes(
                    entry.mime.as_ref(),
                    &Bytes::from_owned(bytes.clone()),
                )),
                Value::Image(image) => {
                    providers.push(gdk::ContentProvider::for_value(&image.texture().to_value()))
                }
                Value::Paths(paths) => {
                    let files = paths
                        .iter()
                        .map(|path| gio::File::for_path(Path::new(path)))
                        .collect::<Vec<_>>();
                    if !files.is_empty() {
                        let list = gdk::FileList::from_array(&files);
                        providers.push(gdk::ContentProvider::for_value(&list.to_value()));
                    }
                }
            }
        }
        match providers.len() {
            0 => Ok(None),
            1 => Ok(providers.pop()),
            _ => Ok(Some(gdk::ContentProvider::new_union(&providers))),
        }
    }
}

impl IntoIterator for Data {
    type IntoIter = DataIntoIter;
    type Item = Result<DataEntry<Image>>;

    fn into_iter(self) -> Self::IntoIter {
        DataIntoIter(
            self.entries
                .into_iter()
                .map(Entry::into_entry)
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

fn decode_uri_list(bytes: &[u8]) -> Option<Vec<OsString>> {
    let text = std::str::from_utf8(bytes).ok()?;
    let uris = text
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .filter_map(|uri| gio::File::for_uri(uri).path())
        .map(PathBuf::into_os_string)
        .collect::<Vec<_>>();
    if uris.is_empty() { None } else { Some(uris) }
}

type MimeRead = Pin<
    Box<
        dyn Future<Output = std::result::Result<(gio::InputStream, glib::GString), glib::Error>>
            + 'static,
    >,
>;
type ValueRead =
    Pin<Box<dyn Future<Output = std::result::Result<glib::Value, glib::Error>> + 'static>>;

trait ContentSource {
    fn formats(&self) -> gdk::ContentFormats;
    fn read_mime(&self, mime: &str) -> MimeRead;
    fn read_value(&self, ty: glib::types::Type) -> ValueRead;
}

macro_rules! impl_content_source {
    ($ty:ty) => {
        impl ContentSource for $ty {
            fn formats(&self) -> gdk::ContentFormats {
                <$ty>::formats(self)
            }

            fn read_mime(&self, mime: &str) -> MimeRead {
                self.read_future(&[mime], glib::Priority::DEFAULT)
            }

            fn read_value(&self, ty: glib::types::Type) -> ValueRead {
                self.read_value_future(ty, glib::Priority::DEFAULT)
            }
        }
    };
}

impl_content_source!(gdk::Clipboard);
impl_content_source!(gdk::Drop);

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
