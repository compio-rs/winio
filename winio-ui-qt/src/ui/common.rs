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

impl TryFrom<String> for QString {
    type Error = cxx::Exception;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.as_str().try_into()
    }
}

impl TryFrom<&str> for QString {
    type Error = cxx::Exception;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        unsafe { ffi::from_utf8(value.as_ptr(), value.len()) }
    }
}

impl TryFrom<QString> for String {
    type Error = cxx::Exception;

    fn try_from(value: QString) -> Result<Self, Self::Error> {
        (&value).try_into()
    }
}

impl TryFrom<&QString> for String {
    type Error = cxx::Exception;

    fn try_from(value: &QString) -> Result<Self, Self::Error> {
        let utf16_ptr = value.utf16()?;
        let len = ffi::string_len(value);
        let slice = unsafe { std::slice::from_raw_parts(utf16_ptr, len) };
        Ok(String::from_utf16_lossy(slice))
    }
}

impl Debug for QString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match String::try_from(self) {
            Ok(s) => s.fmt(f),
            Err(_) => write!(f, "<invalid QString>"),
        }
    }
}

#[repr(C)]
pub struct QUrl {
    _space: MaybeUninit<usize>,
}

unsafe impl ExternType for QUrl {
    type Id = type_id!("QUrl");
    type Kind = cxx::kind::Trivial;
}

impl TryFrom<&QString> for QUrl {
    type Error = cxx::Exception;

    fn try_from(value: &QString) -> Result<Self, Self::Error> {
        ffi::new_url(value)
    }
}

impl TryFrom<QString> for QUrl {
    type Error = cxx::Exception;

    fn try_from(value: QString) -> Result<Self, Self::Error> {
        (&value).try_into()
    }
}

impl TryFrom<&QUrl> for QString {
    type Error = cxx::Exception;

    fn try_from(value: &QUrl) -> Result<Self, Self::Error> {
        ffi::url_to_qstring(value)
    }
}

impl TryFrom<QUrl> for QString {
    type Error = cxx::Exception;

    fn try_from(value: QUrl) -> Result<Self, Self::Error> {
        (&value).try_into()
    }
}

impl TryFrom<&str> for QUrl {
    type Error = cxx::Exception;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        QString::try_from(value).and_then(|qstring| qstring.try_into())
    }
}

impl TryFrom<&QUrl> for String {
    type Error = cxx::Exception;

    fn try_from(value: &QUrl) -> Result<Self, Self::Error> {
        QString::try_from(value).and_then(|qstring| qstring.try_into())
    }
}

impl TryFrom<QUrl> for String {
    type Error = cxx::Exception;

    fn try_from(value: QUrl) -> Result<Self, Self::Error> {
        (&value).try_into()
    }
}

#[cxx::bridge]
mod ffi {
    unsafe extern "C++-unwind" {
        include!("winio-ui-qt/src/ui/common.hpp");

        type QString = super::QString;
        type QUrl = super::QUrl;

        #[cxx_name = "new_string_utf8"]
        unsafe fn from_utf8(p: *const u8, size: usize) -> Result<QString>;

        fn utf16(self: &QString) -> Result<*const u16>;
        fn string_len(s: &QString) -> usize;
        fn string_drop(s: Pin<&mut QString>);

        fn new_url(s: &QString) -> Result<QUrl>;

        fn url_to_qstring(url: &QUrl) -> Result<QString>;
    }
}
