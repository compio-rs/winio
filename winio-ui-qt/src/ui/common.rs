use std::{fmt::Debug, mem::MaybeUninit, pin::Pin};

use cxx::{ExternType, type_id};

#[repr(C)]
pub struct QString {
    #[cfg(qtver = "6")]
    _data: MaybeUninit<[usize; 3]>,
    #[cfg(qtver = "5")]
    _data: MaybeUninit<[usize; 1]>,
}

unsafe impl ExternType for QString {
    type Id = type_id!("QString");
    type Kind = cxx::kind::Trivial;
}

impl Drop for QString {
    fn drop(&mut self) {
        unsafe {
            ffi::string_drop(Pin::new_unchecked(self));
        }
    }
}

impl From<String> for QString {
    fn from(value: String) -> Self {
        value.as_str().into()
    }
}

impl From<&str> for QString {
    fn from(value: &str) -> Self {
        unsafe { ffi::from_utf8(value.as_ptr(), value.len()) }
    }
}

impl From<QString> for String {
    fn from(value: QString) -> Self {
        (&value).into()
    }
}

impl From<&QString> for String {
    fn from(value: &QString) -> Self {
        String::from_utf16_lossy(unsafe {
            std::slice::from_raw_parts(value.utf16(), ffi::string_len(value))
        })
    }
}

impl Debug for QString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        String::from(self).fmt(f)
    }
}

#[cxx::bridge]
mod ffi {
    unsafe extern "C++-unwind" {
        include!("winio-ui-qt/src/ui/common.hpp");

        type QString = super::QString;

        #[cxx_name = "new_string_utf8"]
        unsafe fn from_utf8(p: *const u8, size: usize) -> QString;

        fn utf16(self: &QString) -> *const u16;
        fn string_len(s: &QString) -> usize;
        fn string_drop(s: Pin<&mut QString>);
    }
}
