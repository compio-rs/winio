use std::{cell::Cell, ffi::OsStr, fmt::Debug};

use cxx::UniquePtr;
use winio_callback::Callback;
use winio_pollable::GlobalRuntime;

use crate::{Result, common::QString, platform::Data};

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

    pub fn read(&mut self) -> Result<Option<Data>> {
        let inner = ffi::clipboard_read()?;
        if inner.is_null() {
            return Ok(None);
        }
        let data = Data::from_owned(inner)?;
        if data.formats()?.is_empty() {
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

    pub fn write_text(&mut self, text: impl AsRef<str>) -> Result<()> {
        let text = QString::try_from(text.as_ref())?;
        self.with_write(|| {
            ffi::clipboard_write_text(&text)?;
            Ok(())
        })
    }

    pub fn write_rtf(&mut self, rtf: impl AsRef<str>) -> Result<()> {
        let rtf = QString::try_from(rtf.as_ref())?;
        self.with_write(|| {
            ffi::clipboard_write_rtf(&rtf)?;
            Ok(())
        })
    }

    pub fn write_html(&mut self, html: impl AsRef<str>) -> Result<()> {
        let html = QString::try_from(html.as_ref())?;
        self.with_write(|| {
            ffi::clipboard_write_html(&html)?;
            Ok(())
        })
    }

    pub fn write_image(&mut self, image: &crate::Image) -> Result<()> {
        self.with_write(|| {
            ffi::clipboard_write_image(image.as_qimage())?;
            Ok(())
        })
    }

    pub fn write_uris(&mut self, uris: impl IntoIterator<Item = impl AsRef<OsStr>>) -> Result<()> {
        let mut uris = uris
            .into_iter()
            .map(|uri| QString::try_from(uri.as_ref().to_string_lossy().as_ref()))
            .collect::<std::result::Result<Vec<_>, _>>()?;
        self.with_write(|| {
            ffi::clipboard_write_uris(&mut uris)?;
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
        type QImage = crate::platform::QImage;
        type QMimeData = crate::platform::QMimeData;
        type QString = crate::common::QString;

        unsafe fn clipboard_connect_changed(
            callback: unsafe fn(*const u8),
            data: *const u8,
        ) -> Result<UniquePtr<ClipboardContext>>;

        fn clipboard_read() -> Result<UniquePtr<QMimeData>>;
        fn clipboard_write(data: &QMimeData) -> Result<()>;
        fn clipboard_clear() -> Result<()>;
        fn clipboard_write_text(text: &QString) -> Result<()>;
        fn clipboard_write_rtf(rtf: &QString) -> Result<()>;
        fn clipboard_write_html(html: &QString) -> Result<()>;
        fn clipboard_write_image(image: &QImage) -> Result<()>;
        fn clipboard_write_uris(uris: &mut [QString]) -> Result<()>;
    }
}
