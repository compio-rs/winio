use std::{ffi::OsString, io::Cursor};

use futures_util::Stream;
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
use windows_core::{HSTRING, Interface, WIN32_ERROR};
use windows_sys::Win32::Foundation::ERROR_INVALID_DATA;
use winio_primitive::{DataEntry, DataFormat};

use crate::{Image, Result};

const UNKNOWN_FORMAT: &str = "application/vnd.winio.unknown";

#[derive(Debug)]
pub struct Data {
    package: DataPackage,
}

impl Data {
    pub fn new() -> Result<Self> {
        Ok(Self {
            package: DataPackage::new()?,
        })
    }

    pub async fn insert(&mut self, entry: DataEntry<Image>) -> Result<()> {
        match &entry {
            DataEntry::Text(text) => self.package.SetText(&HSTRING::from(text))?,
            DataEntry::Html(html) => self.package.SetHtmlFormat(&HSTRING::from(html))?,
            DataEntry::Image(image) => {
                let reference = image_reference(image).await?;
                self.package.SetBitmap(&reference)?;
            }
            DataEntry::Paths(paths) => {
                let items = storage_items(paths).await?;
                self.package.SetStorageItems(&items, false)?;
            }
            DataEntry::Unknown(bytes) => {
                let reference = stream_reference(bytes).await?;
                self.package
                    .SetData(&HSTRING::from(UNKNOWN_FORMAT), &reference)?;
            }
        }
        Ok(())
    }

    pub fn formats(&self) -> Result<impl Iterator<Item = DataFormat>> {
        let formats = self
            .package
            .GetView()?
            .AvailableFormats()?
            .into_iter()
            .map(|format| format_of(&format.to_string_lossy()))
            .collect::<Vec<_>>();
        Ok(formats.into_iter())
    }

    pub fn into_stream(self) -> impl Stream<Item = Result<DataEntry<Image>>> + Unpin {
        Box::pin(async_stream::stream! {
            let view = match self.package.GetView() {
                Ok(view) => view,
                Err(error) => {
                    yield Err(error);
                    return;
                }
            };
            let formats = match view.AvailableFormats() {
                Ok(formats) => formats,
                Err(error) => {
                    yield Err(error);
                    return;
                }
            };
            for format in formats.into_iter() {
                yield read_format(&view, &format).await;
            }
        })
    }

    pub(crate) fn package(&self) -> &DataPackage {
        &self.package
    }

    pub(crate) async fn from_data_package_view(view: &DataPackageView) -> Result<Option<Self>> {
        if let Ok(package) = view.cast::<DataPackage>() {
            return Ok(Some(Self { package }));
        }
        let data = Self::new()?;
        let formats = view.AvailableFormats()?;
        for format in formats.into_iter() {
            match format.to_string_lossy().as_str() {
                "Text" => {
                    let text = view.GetTextAsync()?.await?.to_string_lossy();
                    data.package.SetText(&HSTRING::from(text))?;
                }
                "HTML Format" => {
                    let html = view.GetHtmlFormatAsync()?.await?.to_string_lossy();
                    data.package.SetHtmlFormat(&HSTRING::from(html))?;
                }
                "RTF" => {
                    let rtf = view.GetRtfAsync()?.await?.to_string_lossy();
                    data.package.SetRtf(&HSTRING::from(rtf))?;
                }
                "Bitmap" => {
                    let reference = view.GetBitmapAsync()?.await?;
                    data.package.SetBitmap(&reference)?;
                }
                "StorageItems" => {
                    let items = view.GetStorageItemsAsync()?.await?;
                    let items = items.into_iter().collect::<Vec<_>>();
                    data.package.SetStorageItems(
                        &IIterable::from(items.into_iter().map(Some).collect::<Vec<_>>()),
                        false,
                    )?;
                }
                _ => {
                    let value = view.GetDataAsync(&format)?.await?;
                    data.package.SetData(&format, &value)?;
                }
            }
        }
        if data.formats()?.next().is_none() {
            Ok(None)
        } else {
            Ok(Some(data))
        }
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

async fn read_format(view: &DataPackageView, format: &HSTRING) -> Result<DataEntry<Image>> {
    match format.to_string_lossy().as_str() {
        "Text" => Ok(DataEntry::Text(
            view.GetTextAsync()?.await?.to_string_lossy(),
        )),
        "HTML Format" => Ok(DataEntry::Html(
            view.GetHtmlFormatAsync()?.await?.to_string_lossy(),
        )),
        "RTF" => Ok(DataEntry::Unknown(
            view.GetRtfAsync()?.await?.to_string_lossy().into_bytes(),
        )),
        "Bitmap" => read_image(view.GetBitmapAsync()?.await?).await,
        "StorageItems" => Ok(DataEntry::Paths(
            view.GetStorageItemsAsync()?
                .await?
                .into_iter()
                .filter_map(|item| item.Path().ok())
                .map(|path| OsString::from(path.to_string_lossy()))
                .collect(),
        )),
        _ => match view.GetDataAsync(format)?.await {
            Ok(value) => match value.cast::<RandomAccessStreamReference>() {
                Ok(reference) => read_reference(reference).await.map(DataEntry::Unknown),
                Err(_) => Ok(DataEntry::Unknown(Vec::new())),
            },
            Err(error) => Err(error),
        },
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
    WIN32_ERROR(ERROR_INVALID_DATA).to_hresult().into()
}
