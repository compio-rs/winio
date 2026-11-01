use std::{
    collections::hash_map::DefaultHasher, ffi::OsString, fmt::Debug, hash::Hasher, io::Cursor,
    path::Path,
};

use base64::prelude::*;
use futures_util::Stream;
use image::ImageFormat;
use jni::{
    Env,
    objects::{Global, JString},
};
use winio_primitive::{DataEntry, DataFormat};

use crate::{
    Error, Image, JCharSequenceExt, Result, current_activity,
    java::{
        android::{
            content::{ClipData, ClipDataItem, ClipboardManager},
            net::Uri,
        },
        lang::File,
    },
    vm_exec,
};

const IMAGE_PREFIX: &str = "winio-image-png:";
const UNKNOWN_PREFIX: &str = "winio-unknown:";

#[derive(Debug)]
pub struct Data {
    clip: Option<Global<ClipData<'static>>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Kind {
    Text,
    Html,
    Path,
    ImageUri,
    ImageData,
    Unknown,
}

impl Data {
    pub fn new() -> Result<Self> {
        Ok(Self { clip: None })
    }

    pub async fn insert(&mut self, entry: DataEntry<Image>) -> Result<()> {
        if matches!(&entry, DataEntry::Paths(paths) if paths.is_empty()) {
            return Ok(());
        }
        let image_bytes = match &entry {
            DataEntry::Image(image) => Some(encode_image(image)?),
            _ => None,
        };

        vm_exec(|env| {
            let (kind, new_clip) = build_clip(env, entry, image_bytes)?;
            let clip = match &self.clip {
                Some(existing) if clip_kind(env, existing)? == kind => {
                    for i in 0..new_clip.get_item_count(env)? {
                        let item = new_clip.get_item_at(env, i)?;
                        existing.add_item(env, &item)?;
                    }
                    env.new_global_ref(existing)?
                }
                _ => env.new_global_ref(new_clip)?,
            };
            self.clip = Some(clip);
            Ok(())
        })
    }

    pub fn formats(&self) -> Result<impl Iterator<Item = DataFormat> + '_> {
        let formats = match &self.clip {
            Some(clip) => vm_exec(|env| clip_formats(env, clip))?,
            None => Vec::new(),
        };
        Ok(formats.into_iter())
    }

    pub fn into_stream(self) -> impl Stream<Item = Result<DataEntry<Image>>> + Unpin {
        let entries = match self.clip {
            Some(clip) => vm_exec(|env| clip_entries(env, &clip)).unwrap_or_else(|e| vec![Err(e)]),
            None => Vec::new(),
        };
        futures_util::stream::iter(entries)
    }

    pub(crate) fn from_clip_data(env: &mut Env<'_>, clip: &ClipData<'_>) -> Result<Option<Self>> {
        if clip.is_null() || clip.get_item_count(env)? == 0 {
            return Ok(None);
        }
        Ok(Some(Self {
            clip: Some(env.new_global_ref(clip)?),
        }))
    }

    pub(crate) fn write_to<'local>(
        &self,
        env: &mut Env<'local>,
        manager: &ClipboardManager<'local>,
    ) -> Result<()> {
        match &self.clip {
            Some(clip) => manager.set_primary_clip(env, clip)?,
            None => manager.clear_primary_clip(env)?,
        }
        Ok(())
    }
}

fn build_clip<'local>(
    env: &mut Env<'local>,
    entry: DataEntry<Image>,
    image_bytes: Option<Vec<u8>>,
) -> Result<(Kind, ClipData<'local>)> {
    let label = env.new_string("")?;
    match entry {
        DataEntry::Text(text) => {
            let text = env.new_string(&text)?;
            let clip = ClipData::new_plain_text(env, &label, &text)?;
            Ok((Kind::Text, clip))
        }
        DataEntry::Html(html) => {
            let html = env.new_string(&html)?;
            let clip = ClipData::new_html_text(env, &label, &html, &html)?;
            Ok((Kind::Html, clip))
        }
        DataEntry::Path(path) => {
            let uri = uri_from_path(env, &path)?;
            let clip = ClipData::new_raw_uri(env, &label, &uri)?;
            Ok((Kind::Path, clip))
        }
        DataEntry::Paths(paths) => {
            let (first, rest) = paths.split_first().ok_or(Error::NotSupported)?;
            let uri = uri_from_path(env, first)?;
            let clip = ClipData::new_raw_uri(env, &label, &uri)?;
            for path in rest {
                let uri = uri_from_path(env, path)?;
                let item = ClipDataItem::with_uri(env, &uri)?;
                clip.add_item(env, &item)?;
            }
            Ok((Kind::Path, clip))
        }
        DataEntry::Image(_) => {
            let bytes = image_bytes.ok_or(Error::NotSupported)?;
            match image_clip(env, &label, &bytes) {
                Ok(clip) => Ok((Kind::ImageUri, clip)),
                Err(_) => {
                    let text = format!("{IMAGE_PREFIX}{}", BASE64_STANDARD.encode(bytes));
                    let text = env.new_string(&text)?;
                    let clip = ClipData::new_plain_text(env, &label, &text)?;
                    Ok((Kind::ImageData, clip))
                }
            }
        }
        DataEntry::Unknown(bytes) => {
            let text = format!("{UNKNOWN_PREFIX}{}", BASE64_STANDARD.encode(bytes));
            let text = env.new_string(&text)?;
            let clip = ClipData::new_plain_text(env, &label, &text)?;
            Ok((Kind::Unknown, clip))
        }
    }
}

fn clip_kind(env: &mut Env<'_>, clip: &ClipData<'_>) -> Result<Kind> {
    if clip.get_item_count(env)? == 0 {
        return Ok(Kind::Unknown);
    }
    let is_image = clip_is_image(env, clip)?;
    let item = clip.get_item_at(env, 0)?;
    item_kind(env, &item, is_image)
}

fn item_kind(env: &mut Env<'_>, item: &ClipDataItem<'_>, is_image: bool) -> Result<Kind> {
    let html = item.get_html_text(env)?;
    if !html.is_null() && !html.try_to_string(env)?.is_empty() {
        return Ok(Kind::Html);
    }
    let uri = item.get_uri(env)?;
    if !uri.is_null() && !uri.to_string(env)?.try_to_string(env)?.is_empty() {
        return Ok(if is_image { Kind::ImageUri } else { Kind::Path });
    }
    let text = item.get_text(env)?;
    if !text.is_null() {
        let text = text.try_to_string(env)?;
        if text.starts_with(IMAGE_PREFIX) {
            return Ok(Kind::ImageData);
        }
        if text.starts_with(UNKNOWN_PREFIX) {
            return Ok(Kind::Unknown);
        }
        if !text.is_empty() {
            return Ok(Kind::Text);
        }
    }
    Ok(Kind::Unknown)
}

fn clip_formats(env: &mut Env<'_>, clip: &ClipData<'_>) -> Result<Vec<DataFormat>> {
    let is_image = clip_is_image(env, clip)?;
    let mut formats = Vec::new();
    for i in 0..clip.get_item_count(env)? {
        let item = clip.get_item_at(env, i)?;
        let mut push = |format| {
            if !formats.contains(&format) {
                formats.push(format);
            }
        };

        let html = item.get_html_text(env)?;
        if !html.is_null() && !html.try_to_string(env)?.is_empty() {
            push(DataFormat::Html);
        }

        let uri = item.get_uri(env)?;
        if !uri.is_null() && !uri.to_string(env)?.try_to_string(env)?.is_empty() {
            push(if is_image {
                DataFormat::Image
            } else {
                DataFormat::Path
            });
        }

        let text = item.get_text(env)?;
        if !text.is_null() {
            let text = text.try_to_string(env)?;
            if text.starts_with(IMAGE_PREFIX) {
                push(DataFormat::Image);
            } else if text.starts_with(UNKNOWN_PREFIX) {
                push(DataFormat::Unknown);
            } else if !text.is_empty() {
                push(DataFormat::Text);
            }
        }
    }
    Ok(formats)
}

fn clip_entries(env: &mut Env<'_>, clip: &ClipData<'_>) -> Result<Vec<Result<DataEntry<Image>>>> {
    let is_image = clip_is_image(env, clip)?;
    let mut entries = Vec::new();
    for i in 0..clip.get_item_count(env)? {
        let item = clip.get_item_at(env, i)?;

        let html = item.get_html_text(env)?;
        if !html.is_null() {
            let html = html.try_to_string(env)?;
            if !html.is_empty() {
                entries.push(Ok(DataEntry::Html(html)));
            }
        }

        let uri = item.get_uri(env)?;
        if !uri.is_null() {
            let uri_string = uri.to_string(env)?.try_to_string(env)?;
            if !uri_string.is_empty() {
                if is_image {
                    entries.push(image_from_uri(env, &uri).map(DataEntry::Image));
                } else {
                    entries.push(Ok(DataEntry::Path(OsString::from(uri_string))));
                }
            }
        }

        let text = item.get_text(env)?;
        if !text.is_null() {
            let text = text.try_to_string(env)?;
            if let Some(encoded) = text.strip_prefix(IMAGE_PREFIX) {
                entries.push(
                    BASE64_STANDARD
                        .decode(encoded)
                        .map_err(|_| Error::NotSupported)
                        .and_then(|bytes| decode_image(&bytes))
                        .map(DataEntry::Image),
                );
            } else if let Some(encoded) = text.strip_prefix(UNKNOWN_PREFIX) {
                entries.push(
                    BASE64_STANDARD
                        .decode(encoded)
                        .map(DataEntry::Unknown)
                        .map_err(|_| Error::NotSupported),
                );
            } else if !text.is_empty() {
                entries.push(Ok(DataEntry::Text(text)));
            }
        }
    }
    Ok(entries)
}

fn clip_is_image(env: &mut Env<'_>, clip: &ClipData<'_>) -> Result<bool> {
    let description = clip.get_description(env)?;
    for i in 0..description.get_mime_type_count(env)? {
        let mime = description.get_mime_type(env, i)?;
        if !mime.is_null() && mime.try_to_string(env)?.starts_with("image/") {
            return Ok(true);
        }
    }
    Ok(false)
}

fn image_from_uri(env: &mut Env<'_>, uri: &Uri<'_>) -> Result<Image> {
    let act = current_activity(env)?;
    let resolver = act.as_context().get_content_resolver(env)?;
    let stream = resolver.open_input_stream(env, uri)?;
    let buffer = env.new_byte_array(64 * 1024)?;
    let mut bytes = Vec::new();
    loop {
        let n = stream.read(env, &buffer)?;
        if n <= 0 {
            break;
        }
        let mut chunk = vec![0i8; n as usize];
        buffer.get_region(env, 0, &mut chunk)?;
        bytes.extend(chunk.into_iter().map(|b| b as u8));
    }
    stream.close(env)?;
    decode_image(&bytes)
}

fn image_clip<'local>(
    env: &mut Env<'local>,
    label: &JString<'_>,
    bytes: &[u8],
) -> Result<ClipData<'local>> {
    let act = current_activity(env)?;
    let context = act.as_context();
    let package = context.get_package_name(env)?.try_to_string(env)?;
    let cache = context
        .get_cache_dir(env)?
        .get_absolute_path(env)?
        .try_to_string(env)?;

    let mut hasher = DefaultHasher::new();
    hasher.write(bytes);
    let filename = format!("image-{:016x}.png", hasher.finish());
    let dir = Path::new(&cache).join("winio-images");
    std::fs::create_dir_all(&dir)?;
    std::fs::write(dir.join(&filename), bytes)?;

    let uri = format!("content://{package}.winio.fileprovider/winio_images/{filename}");
    let uri = env.new_string(&uri)?;
    let uri = Uri::parse(env, &uri)?;
    let resolver = context.get_content_resolver(env)?;
    let clip = ClipData::new_uri(env, &resolver, label, &uri)?;
    if !clip_is_image(env, &clip)? {
        return Err(Error::NotSupported);
    }
    Ok(clip)
}

fn uri_from_path<'local>(env: &mut Env<'local>, path: &OsString) -> Result<Uri<'local>> {
    let path = env.new_string(path.to_string_lossy())?;
    let file = File::new(env, path)?;
    Ok(Uri::from_file(env, &file)?)
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
