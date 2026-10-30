use std::{cell::Cell, fmt::Debug};

use cxx::UniquePtr;
use winio_callback::Callback;
use winio_pollable::GlobalRuntime;

use crate::{Result, platform::Data};

pub struct Clipboard {
    _context: UniquePtr<ffi::ClipboardContext>,
    inner: Box<ClipboardInner>,
}

struct ClipboardInner {
    on_changed: Callback<()>,
    writing: Cell<bool>,
}

impl Clipboard {
    pub fn new() -> Result<Self> {
        let inner = Box::new(ClipboardInner {
            on_changed: Callback::new(),
            writing: Cell::new(false),
        });
        let context = unsafe {
            ffi::clipboard_connect_changed(
                Self::on_changed,
                &*inner as *const ClipboardInner as *const u8,
            )?
        };
        Ok(Self {
            _context: context,
            inner,
        })
    }

    pub async fn read(&mut self) -> Result<Option<Data>> {
        let inner = ffi::clipboard_read()?;
        if inner.is_null() {
            return Ok(None);
        }
        let data = Data::from_owned(inner)?;
        if data.is_empty() {
            Ok(None)
        } else {
            Ok(Some(data))
        }
    }

    pub fn write(&mut self, data: &Data) -> Result<()> {
        self.with_write(|| {
            ffi::clipboard_write(data.qmime_data())?;
            Ok(())
        })
    }

    pub fn clear(&mut self) -> Result<()> {
        self.with_write(|| {
            ffi::clipboard_clear()?;
            Ok(())
        })
    }

    pub async fn wait_change(&self) {
        self.inner.on_changed.wait().await
    }

    fn on_changed(c: *const u8) {
        let c = c as *const ClipboardInner;
        if let Some(c) = unsafe { c.as_ref() }
            && !c.writing.get()
        {
            c.on_changed.signal::<GlobalRuntime>(());
        }
    }

    fn with_write<T>(&mut self, f: impl FnOnce() -> Result<T>) -> Result<T> {
        self.inner.writing.set(true);
        let _guard = WriteGuard(&self.inner.writing);
        f()
    }
}

struct WriteGuard<'a>(&'a Cell<bool>);

impl Drop for WriteGuard<'_> {
    fn drop(&mut self) {
        self.0.set(false);
    }
}

impl Debug for Clipboard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Clipboard").finish_non_exhaustive()
    }
}

#[cxx::bridge]
mod ffi {
    unsafe extern "C++-unwind" {
        include!("winio-ui-qt/src/platform/clipboard.hpp");

        type ClipboardContext;
        type QMimeData = crate::platform::QMimeData;

        unsafe fn clipboard_connect_changed(
            callback: unsafe fn(*const u8),
            data: *const u8,
        ) -> Result<UniquePtr<ClipboardContext>>;

        fn clipboard_read() -> Result<UniquePtr<QMimeData>>;
        fn clipboard_write(data: &QMimeData) -> Result<()>;
        fn clipboard_clear() -> Result<()>;
    }
}
