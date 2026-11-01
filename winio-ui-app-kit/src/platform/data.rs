use std::{ffi::OsString, fmt::Debug};

use futures_util::Stream;
use itertools::Itertools;
use objc2::{rc::Retained, runtime::ProtocolObject};
use objc2_app_kit::{NSPasteboard, NSPasteboardItem, NSPasteboardWriting};
use objc2_foundation::{NSData, NSMutableArray, NSString, NSURL, ns_string};
use winio_primitive::{DataEntry, DataFormat};

use crate::{
    Error, Image, Result, catch, decode_image, encode_png, file_url_uti, format_of, from_nsstring,
    html_uti, path_from_file_url, png_uti, text_uti, tiff_uti, unknown_uti,
};

pub(crate) fn drag_types() -> [&'static NSString; 14] {
    [
        text_uti(),
        html_uti(),
        png_uti(),
        tiff_uti(),
        file_url_uti(),
        ns_string!("public.url"),
        ns_string!("public.rtf"),
        ns_string!("com.apple.flat-rtfd"),
        ns_string!("com.adobe.pdf"),
        ns_string!("public.utf8-tab-separated-values-text"),
        ns_string!("com.apple.cocoa.pasteboard.color"),
        ns_string!("com.apple.cocoa.pasteboard.sound"),
        ns_string!("com.apple.cocoa.pasteboard.multiple-text-selection"),
        unknown_uti(),
    ]
}

#[derive(Debug)]
pub struct Data {
    items: Retained<NSMutableArray<NSPasteboardItem>>,
}

impl Data {
    pub fn new() -> Result<Self> {
        let items = catch(|| NSMutableArray::from_retained_slice(&[]))?;
        Ok(Self { items })
    }

    pub async fn insert(&mut self, entry: DataEntry<Image>) -> Result<()> {
        catch(|| {
            match entry {
                DataEntry::Text(text) => {
                    let item = NSPasteboardItem::new();
                    if !item.setString_forType(&NSString::from_str(&text), text_uti()) {
                        return Err(Error::NotSupported);
                    }
                    self.items.addObject(&item);
                }
                DataEntry::Html(html) => {
                    let item = NSPasteboardItem::new();
                    if !item.setString_forType(&NSString::from_str(&html), html_uti()) {
                        return Err(Error::NotSupported);
                    }
                    self.items.addObject(&item);
                }
                DataEntry::Image(image) => {
                    let bytes = encode_png(image.drawing_image())?;
                    let item = NSPasteboardItem::new();
                    if !item.setData_forType(&NSData::with_bytes(&bytes), png_uti()) {
                        return Err(Error::NotSupported);
                    }
                    self.items.addObject(&item);
                }
                DataEntry::Path(path) => self.push_path(&path)?,
                DataEntry::Paths(paths) => {
                    for path in &paths {
                        self.push_path(path)?;
                    }
                }
                DataEntry::Unknown(bytes) => {
                    let item = NSPasteboardItem::new();
                    if !item.setData_forType(&NSData::with_bytes(&bytes), unknown_uti()) {
                        return Err(Error::NotSupported);
                    }
                    self.items.addObject(&item);
                }
            }
            Ok(())
        })
        .flatten()
    }

    pub fn formats(&self) -> Result<impl Iterator<Item = DataFormat> + '_> {
        catch(|| {
            self.items
                .iter()
                .flat_map(|item| item.types())
                .map(|uti| format_of(&uti))
                .unique()
        })
    }

    pub fn into_stream(self) -> impl Stream<Item = Result<DataEntry<Image>>> + Unpin {
        let entries = self.items.into_iter().flat_map(|item| {
            item.types()
                .into_iter()
                .map(move |uti| catch(|| read_type(&item, &uti)).flatten())
        });
        futures_util::stream::iter(entries)
    }

    fn push_path(&mut self, path: &OsString) -> Result<()> {
        let url = NSURL::fileURLWithPath(&NSString::from_str(&path.to_string_lossy()));
        let Some(url) = url.absoluteString() else {
            return Err(Error::NotSupported);
        };
        let item = NSPasteboardItem::new();
        if !item.setString_forType(&url, file_url_uti()) {
            return Err(Error::NotSupported);
        }
        self.items.addObject(&item);
        Ok(())
    }

    pub(crate) fn from_pasteboard(pasteboard: &NSPasteboard) -> Result<Option<Self>> {
        catch(|| Self::from_pasteboard_inner(pasteboard)).flatten()
    }

    fn from_pasteboard_inner(pasteboard: &NSPasteboard) -> Result<Option<Self>> {
        let items = NSMutableArray::<NSPasteboardItem>::new();
        if let Some(source) = pasteboard.pasteboardItems() {
            for item in source.iter() {
                let item = copy_item(&item);
                if !item.types().is_empty() {
                    items.addObject(&item);
                }
            }
        } else if let Some(types) = pasteboard.types() {
            let item = NSPasteboardItem::new();
            for uti in types.iter() {
                if let Some(data) = pasteboard.dataForType(&uti) {
                    item.setData_forType(&data, &uti);
                }
            }
            if !item.types().is_empty() {
                items.addObject(&item);
            }
        }
        if items.is_empty() {
            Ok(None)
        } else {
            Ok(Some(Self { items }))
        }
    }

    pub(crate) fn write_to(&self, pasteboard: &NSPasteboard) -> Result<()> {
        catch(|| self.write_to_inner(pasteboard)).flatten()
    }

    fn write_to_inner(&self, pasteboard: &NSPasteboard) -> Result<()> {
        if self.items.is_empty() {
            pasteboard.clearContents();
            return Ok(());
        }

        let objects: Retained<NSMutableArray<ProtocolObject<dyn NSPasteboardWriting>>> =
            NSMutableArray::new();
        for item in self.items.iter() {
            let item = copy_item(&item);
            objects.addObject(&ProtocolObject::from_retained(item));
        }
        pasteboard.clearContents();
        if !pasteboard.writeObjects(&objects) {
            return Err(Error::NotSupported);
        }
        Ok(())
    }
}

fn read_type(item: &NSPasteboardItem, uti: &NSString) -> Result<DataEntry<Image>> {
    match format_of(uti) {
        DataFormat::Text => Ok(DataEntry::Text(read_string(item, uti)?)),
        DataFormat::Html => Ok(DataEntry::Html(read_string(item, uti)?)),
        DataFormat::Image => {
            let data = item.dataForType(uti).ok_or(Error::NotSupported)?;
            Ok(DataEntry::Image(Image::try_from(decode_image(
                &data.to_vec(),
            )?)?))
        }
        DataFormat::Path => {
            let string = item.stringForType(uti).ok_or(Error::NotSupported)?;
            Ok(DataEntry::Path(path_from_file_url(&string)?))
        }
        DataFormat::Unknown => {
            let data = item.dataForType(uti).ok_or(Error::NotSupported)?;
            Ok(DataEntry::Unknown(data.to_vec()))
        }
    }
}

fn read_string(item: &NSPasteboardItem, uti: &NSString) -> Result<String> {
    item.stringForType(uti)
        .map(|string| from_nsstring(&string))
        .ok_or(Error::NotSupported)
}

fn copy_item(item: &NSPasteboardItem) -> Retained<NSPasteboardItem> {
    let new_item = NSPasteboardItem::new();
    for uti in item.types().iter() {
        if let Some(data) = item.dataForType(&uti) {
            new_item.setData_forType(&data, &uti);
        }
    }
    new_item
}
