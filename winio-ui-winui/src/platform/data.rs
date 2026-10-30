use std::{ffi::OsString, fmt::Debug, io::Cursor};

use image::ImageFormat;
use windows::{
    ApplicationModel::DataTransfer::{DataPackage, DataPackageView},
    Storage::{
        IStorageItem, StorageFile,
        Streams::{
            DataReader, DataWriter, IRandomAccessStream, InMemoryRandomAccessStream,
            RandomAccessStreamReference,
        },
    },
};
use windows_collections::IIterable;
use windows_core::{HSTRING, Interface};
use winio_primitive::{DataEntry, DataFormat};

use crate::{Image, Result};

const UNKNOWN_FORMAT: &str = "application/vnd.winio.unknown";

struct Entry {
    format: DataFormat,
    value: Result<DataEntry<Image>>,
}

impl Entry {
    fn new(value: Result<DataEntry<Image>>, format: DataFormat) -> Self {
        Self { format, value }
    }
}

pub struct Data {
    package: Option<DataPackage>,
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
                    .map(|entry| entry.format)
                    .collect::<Vec<_>>(),
            )
            .finish_non_exhaustive()
    }
}

impl Data {
    pub fn new() -> Result<Self> {
        Ok(Self {
            package: None,
            entries: Vec::new(),
        })
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn insert(&mut self, entry: DataEntry<Image>) -> Result<()> {
        let format = entry.format();
        self.package = None;
        self.entries.push(Entry::new(Ok(entry), format));
        Ok(())
    }

    pub(crate) async fn package(&self) -> Result<DataPackage> {
        if let Some(package) = &self.package {
            return Ok(package.clone());
        }
        let package = DataPackage::new()?;
        for entry in &self.entries {
            match entry.value.as_ref().map_err(|_| invalid_data())? {
                DataEntry::Text(text) => package.SetText(&HSTRING::from(text))?,
                DataEntry::Html(html) => package.SetHtmlFormat(&HSTRING::from(html))?,
                DataEntry::Image(image) => {
                    let reference = image_reference(image).await?;
                    package.SetBitmap(&reference)?;
                }
                DataEntry::Paths(paths) => {
                    let items = storage_items(paths).await?;
                    package.SetStorageItems(&items, false)?;
                }
                DataEntry::Unknown(bytes) => {
                    let reference = stream_reference(bytes).await?;
                    package.SetData(&HSTRING::from(UNKNOWN_FORMAT), &reference)?;
                }
            }
        }
        Ok(package)
    }

    pub fn formats(&self) -> Result<impl Iterator<Item = DataFormat> + '_> {
        Ok(self.entries.iter().map(|entry| entry.format))
    }

    pub(crate) async fn from_data_package_view(view: &DataPackageView) -> Result<Option<Self>> {
        let package = DataPackage::new()?;
        let mut entries = Vec::new();
        for format in view.AvailableFormats()? {
            let name = format.to_string_lossy();
            let category = format_of(&name);
            let entry = match name.as_str() {
                "Text" => {
                    let text = view.GetTextAsync()?.await?.to_string_lossy();
                    package.SetText(&HSTRING::from(&text))?;
                    Ok(DataEntry::Text(text))
                }
                "HTML Format" => {
                    let html = view.GetHtmlFormatAsync()?.await?.to_string_lossy();
                    package.SetHtmlFormat(&HSTRING::from(&html))?;
                    Ok(DataEntry::Html(html))
                }
                "RTF" => {
                    let rtf = view.GetRtfAsync()?.await?.to_string_lossy();
                    package.SetRtf(&HSTRING::from(&rtf))?;
                    Ok(DataEntry::Unknown(rtf.into_bytes()))
                }
                "Bitmap" => match view.GetBitmapAsync()?.await {
                    Ok(reference) => {
                        package.SetBitmap(&reference)?;
                        read_image(reference).await
                    }
                    Err(error) => Err(error),
                },
                "StorageItems" => match view.GetStorageItemsAsync()?.await {
                    Ok(items) => {
                        let items = items.into_iter().collect::<Vec<_>>();
                        let paths = items
                            .iter()
                            .filter_map(|item| item.Path().ok())
                            .map(|path| OsString::from(path.to_string_lossy()))
                            .collect::<Vec<_>>();
                        package.SetStorageItems(
                            &IIterable::from(items.into_iter().map(Some).collect::<Vec<_>>()),
                            false,
                        )?;
                        Ok(DataEntry::Paths(paths))
                    }
                    Err(error) => Err(error),
                },
                _ => match view.GetDataAsync(&format)?.await {
                    Ok(value) => {
                        package.SetData(&format, &value)?;
                        match value.cast::<RandomAccessStreamReference>() {
                            Ok(reference) => match read_reference(reference).await {
                                Ok(bytes) => Ok(DataEntry::Unknown(bytes)),
                                Err(error) => Err(error),
                            },
                            Err(_) => Ok(DataEntry::Unknown(Vec::new())),
                        }
                    }
                    Err(error) => Err(error),
                },
            };
            entries.push(Entry::new(entry, category));
        }
        if entries.is_empty() {
            Ok(None)
        } else {
            Ok(Some(Self {
                package: Some(package),
                entries,
            }))
        }
    }
}

impl IntoIterator for Data {
    type IntoIter = DataIntoIter;
    type Item = Result<DataEntry<Image>>;

    fn into_iter(self) -> Self::IntoIter {
        DataIntoIter(self.entries.into_iter())
    }
}

pub struct DataIntoIter(std::vec::IntoIter<Entry>);

impl Iterator for DataIntoIter {
    type Item = Result<DataEntry<Image>>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|entry| entry.value)
    }
}

fn format_of(name: &str) -> DataFormat {
    match name {
        "Text" => DataFormat::Text,
        "HTML Format" => DataFormat::Html,
        "Bitmap" => DataFormat::Image,
        "StorageItems" => DataFormat::Paths,
        _ => DataFormat::Unknown,
    }
}

async fn stream_reference(bytes: &[u8]) -> Result<RandomAccessStreamReference> {
    let stream = InMemoryRandomAccessStream::new()?;
    let writer = DataWriter::CreateDataWriter(&stream)?;
    writer.WriteBytes(bytes)?;
    let buffer = writer.DetachBuffer()?;
    let output = stream.GetOutputStreamAt(0)?;
    output.WriteAsync(&buffer)?.await?;
    stream.Seek(0)?;
    let stream = stream.cast::<IRandomAccessStream>()?;
    RandomAccessStreamReference::CreateFromStream(&stream)
}

async fn image_reference(image: &Image) -> Result<RandomAccessStreamReference> {
    let image = image.to_dynamic_image()?;
    let mut bytes = Vec::new();
    image
        .write_to(&mut Cursor::new(&mut bytes), ImageFormat::Png)
        .map_err(|_| invalid_data())?;
    stream_reference(&bytes).await
}

async fn storage_items(paths: &[OsString]) -> Result<IIterable<IStorageItem>> {
    let mut items = Vec::with_capacity(paths.len());
    for path in paths {
        let path = HSTRING::from(path.to_string_lossy().as_ref());
        let file = StorageFile::GetFileFromPathAsync(&path)?.await?;
        items.push(file.cast::<IStorageItem>()?);
    }
    Ok(IIterable::from(
        items.into_iter().map(Some).collect::<Vec<_>>(),
    ))
}

async fn read_image(reference: RandomAccessStreamReference) -> Result<DataEntry<Image>> {
    let bytes = read_reference(reference).await?;
    let image = image::load_from_memory(&bytes).map_err(|_| invalid_data())?;
    Ok(DataEntry::Image(Image::from_dynamic_image(image)?))
}

async fn read_reference(reference: RandomAccessStreamReference) -> Result<Vec<u8>> {
    let stream = reference.OpenReadAsync()?.await?;
    let size = stream.Size()? as u32;
    let reader = DataReader::CreateDataReader(&stream)?;
    reader.LoadAsync(size)?.await?;
    let mut bytes = vec![0u8; size as usize];
    reader.ReadBytes(&mut bytes)?;
    reader.Close()?;
    Ok(bytes)
}

fn invalid_data() -> crate::Error {
    windows_core::Error::from_hresult(windows_core::HRESULT(0x80070057u32 as i32))
}
