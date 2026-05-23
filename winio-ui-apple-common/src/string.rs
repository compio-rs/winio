use std::ffi::CStr;

use objc2_core_foundation::{CFArray, CFRange, CFString, CFStringBuiltInEncodings};
use objc2_foundation::NSString;

pub trait TollFreeBridge<T>: Sized {
    fn bridge(&self) -> &T {
        unsafe { &*(std::ptr::addr_of!(*self).cast::<T>()) }
    }
}

impl TollFreeBridge<CFString> for NSString {}

impl<T: ?Sized> TollFreeBridge<CFArray> for CFArray<T> {}

#[inline]
pub fn from_nsstring(s: &NSString) -> String {
    let s = s.bridge();
    let len = s.length() as usize;
    if len == 0 {
        return String::new();
    }

    let mut ptr = s.c_string_ptr(CFStringBuiltInEncodings::EncodingUTF8.0);
    if ptr.is_null() {
        ptr = s.c_string_ptr(CFStringBuiltInEncodings::EncodingASCII.0);
    }
    if !ptr.is_null() {
        unsafe {
            let str = CStr::from_ptr(ptr);
            String::from_utf8_unchecked(str.to_bytes().to_vec())
        }
    } else {
        let ptr = s.characters_ptr();
        if !ptr.is_null() {
            String::from_utf16_lossy(unsafe { std::slice::from_raw_parts(ptr, len) })
        } else {
            let mut buffer = Vec::<u16>::with_capacity(len);
            unsafe {
                s.characters(CFRange::new(0, len as isize), buffer.as_mut_ptr());
                buffer.set_len(len);
            }
            String::from_utf16_lossy(&buffer)
        }
    }
}
