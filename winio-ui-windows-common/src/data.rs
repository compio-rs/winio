use std::{
    ffi::OsString,
    os::windows::ffi::OsStringExt,
    ptr::{null_mut, read_unaligned},
};

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
    let mut paths = Vec::new();
    let mut current = Vec::new();
    let mut ended = false;
    for chunk in bytes.as_chunks::<2>().0 {
        let unit = u16::from_ne_bytes(*chunk);
        if unit == 0 {
            if current.is_empty() {
                ended = true;
                break;
            } else {
                paths.push(OsString::from_wide(&current));
                current.clear();
            }
        } else {
            current.push(unit);
        }
    }
    if ended { Some(paths) } else { None }
}

fn parse_ansi(bytes: &[u8]) -> Option<Vec<OsString>> {
    let mut paths = Vec::new();
    for segment in bytes.split(|byte| *byte == 0) {
        if segment.is_empty() {
            break;
        }
        paths.push(OsString::from(decode_ansi(segment, false)?));
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
