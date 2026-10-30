use std::{
    ffi::OsString,
    os::windows::ffi::OsStringExt,
    ptr::{null_mut, read_unaligned},
};

use rclip_cf_html::CfHtmlBuilder;
use windows_sys::Win32::{
    Globalization::{CP_ACP, CP_OEMCP, MultiByteToWideChar},
    UI::Shell::DROPFILES,
};

/// Parse the contents of a `CF_HDROP` / `FileDrop` payload.
pub fn parse_dropfiles(bytes: &[u8]) -> Option<Vec<OsString>> {
    if bytes.len() < size_of::<DROPFILES>() {
        return None;
    }
    let header = unsafe { read_unaligned(bytes.as_ptr().cast::<DROPFILES>()) };
    let offset = header.pFiles as usize;
    if offset > bytes.len() {
        return None;
    }
    let data = &bytes[offset..];
    if header.fWide != 0 {
        parse_wide(data)
    } else {
        parse_ansi(data)
    }
}

fn parse_wide(bytes: &[u8]) -> Option<Vec<OsString>> {
    let owned;
    let units = match bytemuck::try_cast_slice::<u8, u16>(bytes) {
        Ok(units) => units,
        Err(_) => {
            owned = bytes
                .as_chunks::<2>()
                .0
                .iter()
                .map(|chunk| u16::from_ne_bytes([chunk[0], chunk[1]]))
                .collect::<Vec<_>>();
            &owned
        }
    };
    parse_null_terminated(units, |unit| Some(OsString::from_wide(unit)))
}

fn parse_ansi(bytes: &[u8]) -> Option<Vec<OsString>> {
    parse_null_terminated(bytes, |segment| {
        Some(OsString::from(decode_ansi(segment, false)?))
    })
}

fn parse_null_terminated<T: Copy + Default + PartialEq>(
    units: &[T],
    convert: impl Fn(&[T]) -> Option<OsString>,
) -> Option<Vec<OsString>> {
    let mut paths = Vec::new();
    for segment in units.split(|unit| *unit == T::default()) {
        if segment.is_empty() {
            break;
        }
        paths.push(convert(segment)?);
    }
    if paths.is_empty() { None } else { Some(paths) }
}

/// Decode a byte string using the ANSI or OEM code page.
pub fn decode_ansi(bytes: &[u8], oem: bool) -> Option<String> {
    let bytes = bytes.split(|byte| *byte == 0).next().unwrap_or_default();
    if bytes.is_empty() {
        return Some(String::new());
    }
    let code_page = if oem { CP_OEMCP } else { CP_ACP };
    let len = unsafe {
        MultiByteToWideChar(
            code_page,
            0,
            bytes.as_ptr(),
            bytes.len() as i32,
            null_mut(),
            0,
        )
    };
    if len <= 0 {
        return None;
    }
    let mut wide = Vec::with_capacity(len as usize);
    let written = unsafe {
        MultiByteToWideChar(
            code_page,
            0,
            bytes.as_ptr(),
            bytes.len() as i32,
            wide.as_mut_ptr(),
            len,
        )
    };
    if written <= 0 {
        return None;
    }
    unsafe { wide.set_len(written as usize) };
    String::from_utf16(&wide).ok()
}

/// Encode HTML as a `CF_HTML` payload.
pub fn encode_cf_html(html: &str) -> Option<String> {
    let bytes = CfHtmlBuilder::new(html).build().ok()?;
    String::from_utf8(bytes).ok()
}

/// Decode a `CF_HTML` payload and return its fragment.
pub fn decode_cf_html(bytes: &[u8]) -> Option<String> {
    rclip_cf_html::parse(bytes)
        .ok()
        .map(|html| html.fragment.to_string())
}
