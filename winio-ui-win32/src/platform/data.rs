use std::{
    ffi::OsString,
    fmt::Debug,
    io::Cursor,
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

use futures_util::Stream;
use image::ImageFormat;
use widestring::{U16CStr, U16CString};
use windows_core::{Error, Result, WIN32_ERROR};
use windows_sys::Win32::{
    Foundation::{ERROR_INVALID_DATA, GlobalFree, HGLOBAL},
    System::{
        Memory::{GMEM_MOVEABLE, GlobalAlloc, GlobalLock, GlobalSize, GlobalUnlock},
        Ole::{CF_HDROP, CF_UNICODETEXT},
    },
};
use winio_primitive::{DataEntry, DataFormat};
use winio_ui_windows_common::{decode_ansi, parse_dropfiles};

use crate::{
    Image,
    platform::format::{
        build_hdrop, decode_cf_html, decode_dib, encode_cf_html, format_category, format_codec,
        register_format,
    },
};

const UNKNOWN_FORMAT: &str = "application/vnd.winio.unknown";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Codec {
    Raw,
    Utf16,
    Ansi,
    Oem,
    CfHtml,
    Dib,
    HDrop,
}

pub(crate) struct HGlobal(HGLOBAL);

impl HGlobal {
    pub(crate) unsafe fn from_raw(hglobal: HGLOBAL) -> Option<Self> {
        if hglobal.is_null() {
            None
        } else {
            Some(Self(hglobal))
        }
    }

    pub(crate) fn copy_from(hglobal: HGLOBAL) -> Result<Self> {
        let source = LockGuard::new(hglobal)?;
        Self::from_bytes(&source)
    }

    pub(crate) fn from_bytes(bytes: &[u8]) -> Result<Self> {
        unsafe {
            let hglobal = GlobalAlloc(GMEM_MOVEABLE, bytes.len().max(1));
            if hglobal.is_null() {
                return Err(Error::from_thread());
            }
            let this = Self(hglobal);
            let mut guard = LockGuard::new(this.0)?;
            guard[..bytes.len()].copy_from_slice(bytes);
            drop(guard);
            Ok(this)
        }
    }

    pub(crate) fn lock(&self) -> Result<LockGuard<'_>> {
        LockGuard::new(self.0)
    }

    pub(crate) fn raw(&self) -> HGLOBAL {
        self.0
    }
}

impl Drop for HGlobal {
    fn drop(&mut self) {
        unsafe { GlobalFree(self.0) };
    }
}

impl Debug for HGlobal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("HGlobal").field(&self.0).finish()
    }
}

pub(crate) struct LockGuard<'a> {
    hglobal: HGLOBAL,
    ptr: *mut u8,
    len: usize,
    _marker: PhantomData<&'a HGlobal>,
}

impl<'a> LockGuard<'a> {
    fn new(hglobal: HGLOBAL) -> Result<Self> {
        unsafe {
            let len = GlobalSize(hglobal);
            let ptr = if len == 0 {
                std::ptr::dangling_mut()
            } else {
                let ptr = GlobalLock(hglobal) as *mut u8;
                if ptr.is_null() {
                    return Err(Error::from_thread());
                }
                ptr
            };
            Ok(Self {
                hglobal,
                ptr,
                len,
                _marker: PhantomData,
            })
        }
    }
}

impl Deref for LockGuard<'_> {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        unsafe { std::slice::from_raw_parts(self.ptr, self.len) }
    }
}

impl DerefMut for LockGuard<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { std::slice::from_raw_parts_mut(self.ptr, self.len) }
    }
}

impl Drop for LockGuard<'_> {
    fn drop(&mut self) {
        unsafe { GlobalUnlock(self.hglobal) };
    }
}

pub(crate) struct Entry {
    format: u32,
    hglobal: HGlobal,
    codec: Codec,
}

impl Entry {
    fn new(format: u32, hglobal: HGlobal, codec: Codec) -> Self {
        Self {
            format,
            hglobal,
            codec,
        }
    }

    pub(crate) fn format(&self) -> u32 {
        self.format
    }

    pub(crate) fn category(&self) -> DataFormat {
        format_category(self.format)
    }

    pub(crate) fn hglobal(&self) -> (&HGlobal, Codec) {
        (&self.hglobal, self.codec)
    }

    fn into_entry(self) -> Result<DataEntry<Image>> {
        match self.category() {
            DataFormat::Text => match self.codec {
                Codec::Utf16 => {
                    let guard = self.hglobal.lock()?;
                    let str = unsafe { U16CStr::from_ptr_str(guard.as_ptr().cast()) };
                    Ok(DataEntry::Text(str.to_string_lossy()))
                }
                Codec::Ansi | Codec::Oem => Ok(DataEntry::Text(
                    decode_ansi(&self.hglobal.lock()?, self.codec == Codec::Oem)
                        .ok_or_else(invalid_data)?,
                )),
                _ => Err(invalid_data()),
            },
            DataFormat::Html => {
                let guard = self.hglobal.lock()?;
                let str = str::from_utf8(&guard).map_err(|_| invalid_data())?;
                Ok(DataEntry::Html(
                    decode_cf_html(str).ok_or_else(invalid_data)?,
                ))
            }
            DataFormat::Paths => Ok(DataEntry::Paths(
                parse_dropfiles(&self.hglobal.lock()?).ok_or_else(invalid_data)?,
            )),
            DataFormat::Image => {
                let bytes = self.hglobal.lock()?;
                let image = match self.codec {
                    Codec::Dib => decode_dib(&bytes).ok_or_else(invalid_data)?,
                    _ => image::load_from_memory(&bytes).map_err(|_| invalid_data())?,
                };
                Ok(DataEntry::Image(Image::from_dynamic_image(image)?))
            }
            DataFormat::Unknown => Ok(DataEntry::Unknown(self.hglobal.lock()?.to_vec())),
        }
    }
}

/// A multi-format payload used by the Windows clipboard and drop targets.
pub struct Data {
    entries: Vec<Entry>,
}

impl Debug for Data {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Data")
            .field(
                "formats",
                &self.entries.iter().map(Entry::category).collect::<Vec<_>>(),
            )
            .finish()
    }
}

impl Data {
    pub fn new() -> Result<Self> {
        Ok(Self {
            entries: Vec::new(),
        })
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub(crate) fn entries(&self) -> &[Entry] {
        &self.entries
    }

    pub async fn insert(&mut self, entry: DataEntry<Image>) -> Result<()> {
        match entry {
            DataEntry::Text(text) => self.insert_text(text),
            DataEntry::Html(html) => self.insert_html(html),
            DataEntry::Image(image) => self.insert_image(&image),
            DataEntry::Paths(paths) => self.insert_paths(&paths),
            DataEntry::Unknown(bytes) => self.insert_unknown(bytes),
        }
    }

    fn insert_text(&mut self, text: String) -> Result<()> {
        let text = U16CString::from_str_truncate(&text);
        let data = text.into_vec_with_nul();
        let (ptr, len, capacity) = data.into_parts();
        let data = unsafe { Vec::from_parts(ptr.cast(), len * 2, capacity * 2) };
        self.insert_bytes(CF_UNICODETEXT as u32, data, Codec::Utf16)
    }

    fn insert_html(&mut self, html: String) -> Result<()> {
        self.insert_bytes(
            register_format("HTML Format")?,
            encode_cf_html(&html),
            Codec::CfHtml,
        )
    }

    fn insert_image(&mut self, image: &Image) -> Result<()> {
        let mut bytes = Vec::new();
        image
            .as_dynamic_image()
            .write_to(&mut Cursor::new(&mut bytes), ImageFormat::Png)
            .map_err(|_| invalid_data())?;
        self.insert_bytes(register_format("PNG")?, bytes, Codec::Raw)
    }

    fn insert_paths(&mut self, paths: &[OsString]) -> Result<()> {
        let hglobal = build_hdrop(paths)?;
        self.insert_unique(Entry::new(CF_HDROP as u32, hglobal, Codec::HDrop));
        Ok(())
    }

    fn insert_unknown(&mut self, bytes: Vec<u8>) -> Result<()> {
        self.insert_bytes(register_format(UNKNOWN_FORMAT)?, bytes, Codec::Raw)
    }

    fn insert_bytes(&mut self, format: u32, bytes: Vec<u8>, codec: Codec) -> Result<()> {
        let hglobal = HGlobal::from_bytes(&bytes)?;
        self.insert_unique(Entry::new(format, hglobal, codec));
        Ok(())
    }

    pub(crate) fn insert_hglobal(&mut self, format: u32, hglobal: HGlobal) {
        let codec = format_codec(format);
        let entry = Entry::new(format, hglobal, codec);
        self.entries.push(entry);
    }

    fn insert_unique(&mut self, entry: Entry) {
        let category = entry.category();
        if category == DataFormat::Unknown {
            let format = entry.format();
            self.entries.retain(|existing| existing.format() != format);
        } else {
            self.entries
                .retain(|existing| existing.category() != category);
        }
        self.entries.push(entry);
    }

    pub fn formats(&self) -> Result<impl Iterator<Item = DataFormat> + '_> {
        Ok(self.entries.iter().map(Entry::category))
    }

    pub fn into_stream(self) -> impl Stream<Item = Result<DataEntry<Image>>> + Unpin {
        let entries = self
            .entries
            .into_iter()
            .map(Entry::into_entry)
            .collect::<Vec<_>>();
        futures_util::stream::iter(entries)
    }
}

pub(crate) fn invalid_data() -> Error {
    Error::new(WIN32_ERROR(ERROR_INVALID_DATA).to_hresult(), "invalid data")
}
