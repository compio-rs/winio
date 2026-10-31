use std::{ffi::OsString, fmt::Debug, io::Cursor};

use futures_util::Stream;
use image::ImageFormat;
use objc2::{rc::Retained, runtime::ProtocolObject};
use objc2_app_kit::{NSPasteboard, NSPasteboardItem, NSPasteboardWriting};
use objc2_foundation::{NSData, NSMutableArray, NSString, NSURL, ns_string};
use winio_primitive::{DataEntry, DataFormat};

use crate::{Error, Image, Result, catch, from_nsstring};

pub(crate) fn text_uti() -> &'static NSString {
    ns_string!("public.utf8-plain-text")
}

pub(crate) fn html_uti() -> &'static NSString {
    ns_string!("public.html")
}

pub(crate) fn file_url_uti() -> &'static NSString {
    ns_string!("public.file-url")
}

pub(crate) fn png_uti() -> &'static NSString {
    ns_string!("public.png")
}

pub(crate) fn tiff_uti() -> &'static NSString {
    ns_string!("public.tiff")
}

pub(crate) fn unknown_uti() -> &'static NSString {
    ns_string!("com.winio.unknown")
}

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
                    let bytes = encode_image(&image)?;
                    let item = NSPasteboardItem::new();
                    if !item.setData_forType(&NSData::with_bytes(&bytes), png_uti()) {
                        return Err(Error::NotSupported);
                    }
                    self.items.addObject(&item);
                }
                DataEntry::Paths(paths) => {
                    for path in paths {
                        let url =
                            NSURL::fileURLWithPath(&NSString::from_str(&path.to_string_lossy()));
                        let Some(url) = url.absoluteString() else {
                            continue;
                        };
                        let item = NSPasteboardItem::new();
                        if !item.setString_forType(&url, file_url_uti()) {
                            return Err(Error::NotSupported);
                        }
                        self.items.addObject(&item);
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

impl Debug for Data {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let formats = self
            .formats()
            .map(|formats| formats.collect::<Vec<_>>())
            .unwrap_or_default();
        f.debug_struct("Data")
            .field("formats", &formats)
            .finish_non_exhaustive()
    }
}

fn format_of(uti: &NSString) -> DataFormat {
    if uti.isEqualToString(text_uti()) {
        DataFormat::Text
    } else if uti.isEqualToString(html_uti()) {
        DataFormat::Html
    } else if uti.isEqualToString(file_url_uti()) {
        DataFormat::Paths
    } else if uti.isEqualToString(png_uti()) || uti.isEqualToString(tiff_uti()) {
        DataFormat::Image
    } else {
        DataFormat::Unknown
    }
}

fn read_type(item: &NSPasteboardItem, uti: &NSString) -> Result<DataEntry<Image>> {
    match format_of(uti) {
        DataFormat::Text => Ok(DataEntry::Text(read_string(item, uti)?)),
        DataFormat::Html => Ok(DataEntry::Html(read_string(item, uti)?)),
        DataFormat::Image => {
            let data = item.dataForType(uti).ok_or(Error::NotSupported)?;
            Ok(DataEntry::Image(decode_image(&data.to_vec())?))
        }
        DataFormat::Paths => Ok(DataEntry::Paths(vec![read_file_url(item, uti)?])),
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

fn read_file_url(item: &NSPasteboardItem, uti: &NSString) -> Result<OsString> {
    let string = item.stringForType(uti).ok_or(Error::NotSupported)?;
    let url = NSURL::URLWithString(&string).ok_or(Error::NotSupported)?;
    if !url.isFileURL() {
        return Err(Error::NotSupported);
    }
    let path = url.path().ok_or(Error::NotSupported)?;
    Ok(OsString::from(from_nsstring(&path)))
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

fn decode_image(bytes: &[u8]) -> Result<Image> {
    let image = image::load_from_memory(bytes).map_err(|_| Error::NotSupported)?;
    Image::try_from(image)
}

fn encode_image(image: &Image) -> Result<Vec<u8>> {
    let image = image.to_dynamic_image()?;
    let mut bytes = Vec::new();
    image
        .write_to(&mut Cursor::new(&mut bytes), ImageFormat::Png)
        .map_err(|_| Error::NotSupported)?;
    Ok(bytes)
}
