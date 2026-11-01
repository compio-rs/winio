use std::{
    borrow::Cow,
    cell::Cell,
    ffi::{OsStr, OsString},
    io::Cursor,
    time::Duration,
};

use image::ImageFormat;
use objc2::rc::Retained;
use objc2_foundation::{NSInteger, NSString, NSURL, ns_string};
use winio_primitive::{DataEntry, DataFormat};

use crate::{DrawingImage, Error, Result, Timer, from_nsstring};

/// The UTI of plain UTF-8 text.
pub fn text_uti() -> &'static NSString {
    ns_string!("public.utf8-plain-text")
}

/// The UTI of HTML.
pub fn html_uti() -> &'static NSString {
    ns_string!("public.html")
}

/// The UTI of a file URL.
pub fn file_url_uti() -> &'static NSString {
    ns_string!("public.file-url")
}

/// The UTI of a PNG image.
pub fn png_uti() -> &'static NSString {
    ns_string!("public.png")
}

/// The UTI of a TIFF image.
pub fn tiff_uti() -> &'static NSString {
    ns_string!("public.tiff")
}

/// The UTI used for opaque bytes inserted by winio.
pub fn unknown_uti() -> &'static NSString {
    ns_string!("com.winio.unknown")
}

/// Map a pasteboard UTI to a [`DataFormat`].
pub fn format_of(uti: &NSString) -> DataFormat {
    if uti.isEqualToString(text_uti()) {
        DataFormat::Text
    } else if uti.isEqualToString(html_uti()) {
        DataFormat::Html
    } else if uti.isEqualToString(file_url_uti()) {
        DataFormat::Path
    } else if uti.isEqualToString(png_uti()) || uti.isEqualToString(tiff_uti()) {
        DataFormat::Image
    } else {
        DataFormat::Unknown
    }
}

/// Decode image bytes into a [`DrawingImage`].
pub fn decode_image(bytes: &[u8]) -> Result<DrawingImage> {
    let image = image::load_from_memory(bytes).map_err(|_| Error::NotSupported)?;
    DrawingImage::from_image(Cow::Owned(image))
}

/// Encode a [`DrawingImage`] as PNG.
pub fn encode_png(image: &DrawingImage) -> Result<Vec<u8>> {
    let image = image::DynamicImage::try_from(image)?;
    let mut bytes = Vec::new();
    image
        .write_to(&mut Cursor::new(&mut bytes), ImageFormat::Png)
        .map_err(|_| Error::NotSupported)?;
    Ok(bytes)
}

/// Get the local path of a `file://` URL string.
pub fn path_from_file_url(url: &NSString) -> Result<OsString> {
    let url = NSURL::URLWithString(url).ok_or(Error::NotSupported)?;
    if !url.isFileURL() {
        return Err(Error::NotSupported);
    }
    let path = url.path().ok_or(Error::NotSupported)?;
    Ok(OsString::from(from_nsstring(&path)))
}

/// Create a `file://` URL from a local path.
pub fn file_url_from_path(path: &OsStr) -> Result<Retained<NSURL>> {
    Ok(NSURL::fileURLWithPath(&NSString::from_str(
        &path.to_string_lossy(),
    )))
}

/// Convert raw pasteboard data to a [`DataEntry`].
pub fn data_entry(uti: &NSString, bytes: &[u8]) -> Result<DataEntry<DrawingImage>> {
    match format_of(uti) {
        DataFormat::Text => Ok(DataEntry::Text(
            String::from_utf8(bytes.to_vec()).map_err(|_| Error::NotSupported)?,
        )),
        DataFormat::Html => Ok(DataEntry::Html(
            String::from_utf8(bytes.to_vec()).map_err(|_| Error::NotSupported)?,
        )),
        DataFormat::Image => Ok(DataEntry::Image(decode_image(bytes)?)),
        DataFormat::Path => {
            let string = String::from_utf8(bytes.to_vec()).map_err(|_| Error::NotSupported)?;
            Ok(DataEntry::Path(path_from_file_url(&NSString::from_str(
                &string,
            ))?))
        }
        DataFormat::Unknown => Ok(DataEntry::Unknown(bytes.to_vec())),
    }
}

/// The interval to poll a pasteboard change count.
///
/// Neither `NSPasteboard` nor `UIPasteboard` provides a reliable notification
/// for external changes, so the change count is polled.
const CHANGE_INTERVAL: Duration = Duration::from_millis(200);

/// Polls a pasteboard change count and wakes a future when it changes.
pub struct ChangeWatcher {
    timer: Timer,
    count: Cell<NSInteger>,
}

impl ChangeWatcher {
    /// Create a watcher starting at `count`.
    pub fn new(count: NSInteger) -> Result<Self> {
        let mut timer = Timer::new(CHANGE_INTERVAL)?;
        timer.start()?;
        Ok(Self {
            timer,
            count: Cell::new(count),
        })
    }

    /// Record the current count without firing a change.
    pub fn sync(&self, count: NSInteger) {
        self.count.set(count);
    }

    /// Wait until the count returned by `count` differs from the last seen
    /// value.
    pub async fn wait_change<F>(&self, count: F)
    where
        F: Fn() -> Result<NSInteger>,
    {
        loop {
            self.timer.wait().await;
            let Ok(count) = count() else {
                continue;
            };
            if count != self.count.get() {
                self.count.set(count);
                break;
            }
        }
    }
}

impl Drop for ChangeWatcher {
    fn drop(&mut self) {
        let _ = self.timer.stop();
    }
}

impl std::fmt::Debug for ChangeWatcher {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ChangeWatcher").finish_non_exhaustive()
    }
}
