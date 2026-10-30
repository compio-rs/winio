use std::{
    collections::HashSet, ffi::OsString, mem::size_of, os::windows::ffi::OsStringExt, ptr::null_mut,
};

use image::{DynamicImage, RgbaImage};
use widestring::{U16CString, U16Str, U16String, u16str};
use windows_core::{Error, Result};
use windows_sys::Win32::{
    Graphics::Gdi::{BI_RGB, BITMAPINFOHEADER},
    System::{
        DataExchange::{
            GetClipboardData, GetClipboardFormatNameW, RegisterClipboardFormatW, SetClipboardData,
        },
        Ole::{CF_DIB, CF_DIBV5, CF_HDROP, CF_OEMTEXT, CF_TEXT, CF_UNICODETEXT},
    },
    UI::Shell::{DROPFILES, DragQueryFileW},
};
use winio_primitive::DataFormat;

use crate::platform::data::{Codec, Data, HGlobal, invalid_data};

const HTML_FORMAT_NAME: &U16Str = u16str!("HTML Format");
const PNG_FORMAT_NAME: &U16Str = u16str!("PNG");

pub(crate) fn format_codec(format: u32) -> Codec {
    if format == CF_UNICODETEXT as u32 {
        Codec::Utf16
    } else if format == CF_TEXT as u32 {
        Codec::Ansi
    } else if format == CF_OEMTEXT as u32 {
        Codec::Oem
    } else if format == CF_HDROP as u32 {
        Codec::HDrop
    } else if format == CF_DIB as u32 || format == CF_DIBV5 as u32 {
        Codec::Dib
    } else if format_name(format).as_deref() == Some(HTML_FORMAT_NAME) {
        Codec::CfHtml
    } else {
        Codec::Raw
    }
}

pub(crate) fn format_category(format: u32) -> DataFormat {
    if format == CF_UNICODETEXT as u32 || format == CF_TEXT as u32 || format == CF_OEMTEXT as u32 {
        DataFormat::Text
    } else if format == CF_HDROP as u32 {
        DataFormat::Paths
    } else if format == CF_DIB as u32 || format == CF_DIBV5 as u32 {
        DataFormat::Image
    } else {
        let name = format_name(format);
        if name.as_deref() == Some(HTML_FORMAT_NAME) {
            DataFormat::Html
        } else if name.as_deref() == Some(PNG_FORMAT_NAME) {
            DataFormat::Image
        } else {
            DataFormat::Unknown
        }
    }
}

pub(crate) fn copy_clipboard_hglobal(format: u32) -> Result<Option<HGlobal>> {
    let is_hglobal = format == CF_UNICODETEXT as u32
        || format == CF_TEXT as u32
        || format == CF_OEMTEXT as u32
        || format == CF_HDROP as u32
        || format == CF_DIB as u32
        || format == CF_DIBV5 as u32
        || format_name(format).is_some();
    if !is_hglobal {
        return Ok(None);
    }
    unsafe {
        let handle = GetClipboardData(format);
        if handle.is_null() {
            return Ok(None);
        }
        HGlobal::copy_from(handle).map(Some)
    }
}

pub(crate) fn write_clipboard(data: &Data) -> Result<()> {
    let mut written = HashSet::new();
    for entry in data.entries() {
        let format = entry.format();
        if !written.insert(format) {
            continue;
        }
        let (hglobal, _) = entry.hglobal();
        let copy = HGlobal::copy_from(hglobal.raw())?;
        if unsafe { SetClipboardData(format, copy.raw()) }.is_null() {
            return Err(Error::from_thread());
        }
        std::mem::forget(copy);
    }
    Ok(())
}

pub(crate) fn build_hdrop(uris: &[OsString]) -> Result<HGlobal> {
    let mut paths = U16String::new();
    for uri in uris {
        paths.push_os_str(uri);
        paths.push_char(0u8 as _);
    }
    paths.push_char(0u8 as _);
    let paths = paths.into_vec();

    let drop = DROPFILES {
        pFiles: size_of::<DROPFILES>() as u32,
        pt: windows_sys::Win32::Foundation::POINT { x: 0, y: 0 },
        fNC: 0,
        fWide: 1,
    };
    let mut bytes = Vec::with_capacity(size_of::<DROPFILES>() + paths.len() * 2);
    let header =
        unsafe { std::slice::from_raw_parts(&raw const drop as _, size_of::<DROPFILES>()) };
    bytes.extend_from_slice(header);
    for unit in paths {
        bytes.extend_from_slice(&unit.to_ne_bytes());
    }
    HGlobal::from_bytes(&bytes)
}

pub(crate) fn read_hdrop(hglobal: &HGlobal) -> Vec<OsString> {
    let hdrop = hglobal.raw();
    unsafe {
        let count = DragQueryFileW(hdrop, u32::MAX, null_mut(), 0);
        let mut uris = Vec::with_capacity(count as usize);
        for index in 0..count {
            let len = DragQueryFileW(hdrop, index, null_mut(), 0) as usize;
            if len == 0 {
                continue;
            }
            let mut buf = Vec::with_capacity(len + 1);
            let written = DragQueryFileW(hdrop, index, buf.as_mut_ptr(), buf.len() as u32) as usize;
            if written > 0 {
                buf.set_len(written);
                uris.push(OsString::from_wide(&buf[..written]));
            }
        }
        uris
    }
}

fn format_name(format: u32) -> Option<U16String> {
    let mut buf = Vec::with_capacity(256);
    let len = unsafe { GetClipboardFormatNameW(format, buf.as_mut_ptr(), buf.len() as i32) };
    if len <= 0 {
        None
    } else {
        unsafe { buf.set_len(len as usize) };
        Some(U16String::from_vec(buf))
    }
}

pub(crate) fn register_format(name: &str) -> Result<u32> {
    let name = U16CString::from_str(name).map_err(|_| invalid_data())?;
    let format = unsafe { RegisterClipboardFormatW(name.as_ptr()) };
    if format == 0 {
        Err(Error::from_thread())
    } else {
        Ok(format)
    }
}

pub(crate) fn encode_cf_html(html: &str) -> Vec<u8> {
    let body =
        format!("<html><body>\r\n<!--StartFragment-->{html}<!--EndFragment-->\r\n</body></html>");
    let prefix = "<html><body>\r\n<!--StartFragment-->";
    let header_len = 105;
    let start_html = header_len;
    let start_fragment = start_html + prefix.len();
    let end_fragment = start_fragment + html.len();
    let end_html = start_html + body.len();
    let header = format!(
        "Version:0.9\r\nStartHTML:{start_html:010}\r\nEndHTML:{end_html:010}\r\nStartFragment:\
         {start_fragment:010}\r\nEndFragment:{end_fragment:010}\r\n"
    );
    let mut bytes = header.into_bytes();
    bytes.extend_from_slice(body.as_bytes());
    bytes
}

pub(crate) fn decode_cf_html(text: &str) -> Option<String> {
    let start = header_offset(text, "StartFragment:");
    let end = header_offset(text, "EndFragment:");
    if let (Some(start), Some(end)) = (start, end)
        && start <= end
        && end <= text.len()
    {
        return Some(text[start..end].to_string());
    }
    let body = text.split_once("\r\n\r\n").map(|(_, body)| body)?;
    Some(body.to_string())
}

fn header_offset(text: &str, key: &str) -> Option<usize> {
    let line = text.lines().find(|line| line.starts_with(key))?;
    line[key.len()..].trim().parse().ok()
}

const fn channel(value: u32, mask: u32) -> u8 {
    if mask == 0 {
        return 0;
    }
    let shift = mask.trailing_zeros();
    let max = (mask >> shift) as u64;
    ((((value & mask) >> shift) as u64 * 255) / max) as u8
}

pub(crate) fn decode_dib(bytes: &[u8]) -> Option<DynamicImage> {
    const BI_BITFIELDS: u32 = 3;
    const BI_ALPHABITFIELDS: u32 = 6;

    if bytes.len() < size_of::<BITMAPINFOHEADER>() {
        return None;
    }
    let header = unsafe { std::ptr::read_unaligned(bytes.as_ptr().cast::<BITMAPINFOHEADER>()) };
    if header.biSize < size_of::<BITMAPINFOHEADER>() as u32
        || (header.biBitCount != 24 && header.biBitCount != 32)
        || header.biWidth <= 0
        || header.biHeight == 0
    {
        return None;
    }

    let width = header.biWidth as u32;
    let height = header.biHeight.unsigned_abs();
    let top_down = header.biHeight < 0;
    let bpp = (header.biBitCount / 8) as usize;
    let stride = ((width as usize * bpp + 3) & !3) as usize;

    let (offset, red_mask, green_mask, blue_mask, alpha_mask) = match (bpp, header.biCompression) {
        (3, BI_RGB) => (header.biSize as usize, 0, 0, 0, 0),
        (4, BI_RGB) => (
            header.biSize as usize,
            0x00ff0000,
            0x0000ff00,
            0x000000ff,
            0xff000000,
        ),
        (4, BI_BITFIELDS) | (4, BI_ALPHABITFIELDS) => {
            let mask_count = if header.biCompression == BI_ALPHABITFIELDS {
                4
            } else {
                3
            };
            let offset = size_of::<BITMAPINFOHEADER>();
            if bytes.len() < offset + mask_count * size_of::<u32>() {
                return None;
            }
            let red_mask = u32::from_ne_bytes(bytes[offset..offset + 4].try_into().unwrap());
            let green_mask = u32::from_ne_bytes(bytes[offset + 4..offset + 8].try_into().unwrap());
            let blue_mask = u32::from_ne_bytes(bytes[offset + 8..offset + 12].try_into().unwrap());
            let alpha_mask = if mask_count == 4 {
                u32::from_ne_bytes(bytes[offset + 12..offset + 16].try_into().unwrap())
            } else {
                0
            };
            (
                offset + mask_count * 4,
                red_mask,
                green_mask,
                blue_mask,
                alpha_mask,
            )
        }
        _ => return None,
    };

    if bytes.len() < offset + stride * height as usize {
        return None;
    }

    let mut image = RgbaImage::new(width, height);
    for y in 0..height {
        let row = if top_down { y } else { height - 1 - y };
        let row_start = offset + row as usize * stride;
        for x in 0..width {
            let p = row_start + x as usize * bpp;
            let (r, g, b, a) = if bpp == 4 {
                let value = u32::from_ne_bytes(bytes[p..p + 4].try_into().unwrap());
                let a = if alpha_mask == 0 {
                    255
                } else {
                    channel(value, alpha_mask)
                };
                (
                    channel(value, red_mask),
                    channel(value, green_mask),
                    channel(value, blue_mask),
                    a,
                )
            } else {
                (bytes[p + 2], bytes[p + 1], bytes[p], 255)
            };
            image.put_pixel(x, y, image::Rgba([r, g, b, a]));
        }
    }
    Some(DynamicImage::ImageRgba8(image))
}
