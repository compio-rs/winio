use std::{ffi::OsString, fmt::Debug};

use futures_util::Stream;
use itertools::Itertools;
use objc2::{Message, rc::Retained, runtime::AnyObject};
use objc2_foundation::{NSData, NSDictionary, NSMutableArray, NSString, NSURL};
use objc2_ui_kit::{UIImage, UIPasteboard};
use winio_primitive::{DataEntry, DataFormat};

use crate::{
    Error, Image, Result, catch, data_entry, encode_png, file_url_from_path, file_url_uti,
    format_of, from_nsstring, html_uti, png_uti, text_uti, unknown_uti,
};

/// Data transferred through the pasteboard.
#[derive(Debug)]
pub struct Data {
    entries: Vec<Entry>,
}

#[derive(Debug)]
struct Entry {
    uti: Retained<NSString>,
    value: Value,
}

#[derive(Debug)]
enum Value {
    String(Retained<NSString>),
    Data(Retained<NSData>),
    Url(Retained<NSURL>),
    Image(Retained<UIImage>),
}

impl Entry {
    fn new(uti: &'static NSString, value: Value) -> Self {
        Self {
            uti: uti.retain(),
            value,
        }
    }
}

impl Data {
    pub fn new() -> Result<Self> {
        Ok(Self {
            entries: Vec::new(),
        })
    }

    pub async fn insert(&mut self, entry: DataEntry<Image>) -> Result<()> {
        catch(|| {
            match entry {
                DataEntry::Text(text) => self.entries.push(Entry::new(
                    text_uti(),
                    Value::String(NSString::from_str(&text)),
                )),
                DataEntry::Html(html) => self.entries.push(Entry::new(
                    html_uti(),
                    Value::String(NSString::from_str(&html)),
                )),
                DataEntry::Image(image) => {
                    let bytes = encode_png(image.drawing_image())?;
                    self.entries.push(Entry::new(
                        png_uti(),
                        Value::Data(NSData::with_bytes(&bytes)),
                    ));
                }
                DataEntry::Path(path) => self.push_path(&path)?,
                DataEntry::Paths(paths) => {
                    for path in &paths {
                        self.push_path(path)?;
                    }
                }
                DataEntry::Unknown(bytes) => self.entries.push(Entry::new(
                    unknown_uti(),
                    Value::Data(NSData::with_bytes(&bytes)),
                )),
            }
            Ok(())
        })
        .flatten()
    }

    fn push_path(&mut self, path: &OsString) -> Result<()> {
        self.entries.push(Entry::new(
            file_url_uti(),
            Value::Url(file_url_from_path(path)?),
        ));
        Ok(())
    }

    pub fn formats(&self) -> Result<impl Iterator<Item = DataFormat> + '_> {
        catch(|| {
            self.entries
                .iter()
                .map(|entry| format_of(&entry.uti))
                .unique()
        })
    }

    pub(crate) fn from_loaded(
        entries: impl IntoIterator<Item = (Retained<NSString>, Retained<NSData>)>,
    ) -> Self {
        Self {
            entries: entries
                .into_iter()
                .map(|(uti, data)| Entry {
                    uti,
                    value: Value::Data(data),
                })
                .collect(),
        }
    }

    pub fn into_stream(self) -> impl Stream<Item = Result<DataEntry<Image>>> + Unpin {
        let entries = self.entries.into_iter().map(read_entry);
        futures_util::stream::iter(entries)
    }

    pub(crate) fn from_pasteboard(pasteboard: &UIPasteboard) -> Result<Option<Self>> {
        catch(|| {
            let items = unsafe { pasteboard.items() };
            let mut entries = Vec::new();
            for item in items.iter() {
                for uti in item.allKeys().iter() {
                    let Some(value) = item.objectForKey(&uti) else {
                        continue;
                    };
                    let value = if let Some(value) = value.downcast_ref::<NSString>() {
                        Value::String(value.retain())
                    } else if let Some(value) = value.downcast_ref::<NSData>() {
                        Value::Data(value.retain())
                    } else if let Some(value) = value.downcast_ref::<NSURL>() {
                        Value::Url(value.retain())
                    } else if let Some(value) = value.downcast_ref::<UIImage>() {
                        Value::Image(value.retain())
                    } else {
                        continue;
                    };
                    entries.push(Entry { uti, value });
                }
            }
            Ok(if entries.is_empty() {
                None
            } else {
                Some(Self { entries })
            })
        })
        .flatten()
    }

    pub(crate) fn write_to(&self, pasteboard: &UIPasteboard) -> Result<()> {
        catch(|| {
            let items: Retained<NSMutableArray<NSDictionary<NSString, AnyObject>>> =
                NSMutableArray::new();
            for entry in &self.entries {
                let value: Retained<AnyObject> = match &entry.value {
                    Value::String(value) => as_any(value.clone()),
                    Value::Data(value) => as_any(value.clone()),
                    Value::Url(value) => as_any(value.clone()),
                    Value::Image(value) => as_any(value.clone()),
                };
                let item = NSDictionary::from_slices(&[&*entry.uti], &[&*value]);
                items.addObject(&item);
            }
            unsafe { pasteboard.setItems(&items) };
            Ok(())
        })
        .flatten()
    }
}

fn as_any<T: Message>(value: Retained<T>) -> Retained<AnyObject> {
    unsafe { Retained::cast_unchecked(value) }
}

fn read_entry(entry: Entry) -> Result<DataEntry<Image>> {
    let Entry { uti, value } = entry;
    let bytes = match value {
        Value::Data(data) => data.to_vec(),
        Value::String(string) => from_nsstring(&string).into_bytes(),
        Value::Url(url) => {
            let string = url.absoluteString().ok_or(Error::NotSupported)?;
            from_nsstring(&string).into_bytes()
        }
        Value::Image(image) => {
            let data = image.png_representation().ok_or(Error::NotSupported)?;
            return data_entry(png_uti(), &data.to_vec())?.try_map_image(Image::try_from);
        }
    };
    data_entry(&uti, &bytes)?.try_map_image(Image::try_from)
}
