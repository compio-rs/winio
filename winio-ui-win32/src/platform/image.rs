use std::{borrow::Cow, cell::RefCell, collections::BTreeMap, mem::zeroed, ptr::null_mut, rc::Rc};

use image::{DynamicImage, imageops::FilterType};
use windows_core::{Error, HRESULT, WIN32_ERROR};
use windows_sys::Win32::{
    Foundation::{
        E_ABORT, E_ACCESSDENIED, E_BOUNDS, E_FAIL, E_INVALIDARG, E_NOTIMPL, E_OUTOFMEMORY,
        E_UNEXPECTED, ERROR_BUSY, ERROR_FILE_NOT_FOUND, ERROR_INSUFFICIENT_BUFFER,
        ERROR_INVALID_STATE, ERROR_NOT_FOUND, ERROR_NOT_SUPPORTED, HWND,
    },
    Graphics::GdiPlus::{
        self as gdiplus, GdipCreateBitmapFromScan0, GdipCreateHICONFromBitmap, GdipDisposeImage,
        GdiplusShutdown, GdiplusStartup, GdiplusStartupInput, GpBitmap, Status,
    },
    UI::{
        HiDpi::GetSystemMetricsForDpi,
        WindowsAndMessaging::{
            DestroyIcon, HICON, IMAGE_ICON, SM_CXSMICON, SM_CYSMICON, SendMessageW,
        },
    },
};
use winio_primitive::Size;

use super::dpi::{DpiAware, get_dpi_for_window};
use crate::{DrawingContext, DrawingImage, Result};

struct WinIcon(HICON);

impl Drop for WinIcon {
    fn drop(&mut self) {
        unsafe { DestroyIcon(self.0) };
    }
}

fn gdip_status(status: Status) -> Result<()> {
    match status {
        gdiplus::Ok => Ok(()),
        gdiplus::InvalidParameter => Err(Error::from_hresult(HRESULT(E_INVALIDARG))),
        gdiplus::OutOfMemory => Err(Error::from_hresult(HRESULT(E_OUTOFMEMORY))),
        gdiplus::ObjectBusy => Err(Error::from_hresult(WIN32_ERROR(ERROR_BUSY).to_hresult())),
        gdiplus::InsufficientBuffer => Err(Error::from_hresult(
            WIN32_ERROR(ERROR_INSUFFICIENT_BUFFER).to_hresult(),
        )),
        gdiplus::NotImplemented => Err(Error::from_hresult(HRESULT(E_NOTIMPL))),
        gdiplus::Win32Error => Err(Error::from_thread()),
        gdiplus::WrongState => Err(Error::from_hresult(
            WIN32_ERROR(ERROR_INVALID_STATE).to_hresult(),
        )),
        gdiplus::Aborted => Err(Error::from_hresult(HRESULT(E_ABORT))),
        gdiplus::FileNotFound => Err(Error::from_hresult(
            WIN32_ERROR(ERROR_FILE_NOT_FOUND).to_hresult(),
        )),
        gdiplus::ValueOverflow => Err(Error::from_hresult(HRESULT(E_BOUNDS))),
        gdiplus::AccessDenied => Err(Error::from_hresult(HRESULT(E_ACCESSDENIED))),
        gdiplus::FontFamilyNotFound
        | gdiplus::FontStyleNotFound
        | gdiplus::PropertyNotFound
        | gdiplus::ProfileNotFound => Err(Error::from_hresult(
            WIN32_ERROR(ERROR_NOT_FOUND).to_hresult(),
        )),
        gdiplus::UnknownImageFormat
        | gdiplus::NotTrueTypeFont
        | gdiplus::UnsupportedGdiplusVersion
        | gdiplus::PropertyNotSupported => Err(Error::from_hresult(
            WIN32_ERROR(ERROR_NOT_SUPPORTED).to_hresult(),
        )),
        gdiplus::GdiplusNotInitialized => Err(Error::from_hresult(HRESULT(E_UNEXPECTED))),
        _ => Err(Error::from_hresult(HRESULT(E_FAIL))),
    }
}

struct GdipInit {
    token: usize,
}

impl GdipInit {
    pub fn new() -> Result<Self> {
        let mut token = 0;
        let mut input: GdiplusStartupInput = unsafe { zeroed() };
        input.GdiplusVersion = 1;
        gdip_status(unsafe { GdiplusStartup(&mut token, &input, null_mut()) })?;
        Ok(Self { token })
    }
}

impl Drop for GdipInit {
    fn drop(&mut self) {
        unsafe { GdiplusShutdown(self.token) };
    }
}

fn gdip_init() -> Result<()> {
    thread_local! {
        static GDIP_INIT: RefCell<Option<GdipInit>> = const { RefCell::new(None) };
    }
    GDIP_INIT.with_borrow_mut(|init| {
        if init.is_none() {
            *init = Some(GdipInit::new()?);
        }
        Ok(())
    })
}

struct GdipBitmap(*mut GpBitmap);

impl GdipBitmap {
    pub fn new(width: i32, height: i32, stride: i32, format: i32, data: *const u8) -> Result<Self> {
        gdip_init()?;

        let mut bitmap = null_mut();
        gdip_status(unsafe {
            GdipCreateBitmapFromScan0(width, height, stride, format, data, &mut bitmap)
        })?;
        Ok(Self(bitmap))
    }

    pub fn create_hicon(&self) -> Result<WinIcon> {
        let mut icon = null_mut();
        gdip_status(unsafe { GdipCreateHICONFromBitmap(self.0, &mut icon) })?;
        Ok(WinIcon(icon))
    }
}

impl Drop for GdipBitmap {
    fn drop(&mut self) {
        unsafe {
            GdipDisposeImage(self.0.cast());
        }
    }
}

#[derive(Debug, Clone)]
pub struct Image(Rc<DynamicImage>);

impl Image {
    pub fn try_to_drawing(&self, context: &DrawingContext) -> Result<DrawingImage> {
        context.create_image(Cow::Borrowed(&self.0))
    }

    #[allow(non_upper_case_globals)]
    fn to_hicon(&self, (width, height): (i32, i32)) -> Result<WinIcon> {
        let image = self
            .0
            .resize_exact(width as _, height as _, FilterType::Triangle)
            .into_rgba8();

        let mut data = Vec::with_capacity(image.len());
        for pixel in image.pixels() {
            let [r, g, b, a] = pixel.0;
            data.extend_from_slice(&[
                ((b as u32 * a as u32 + 127) / 255) as u8,
                ((g as u32 * a as u32 + 127) / 255) as u8,
                ((r as u32 * a as u32 + 127) / 255) as u8,
                a,
            ]);
        }

        const PixelFormatGDI: i32 = 0x00020000;
        const PixelFormatAlpha: i32 = 0x00040000;
        const PixelFormatCanonical: i32 = 0x00200000;
        const PixelFormat32bppARGB: i32 =
            10 | (32 << 8) | PixelFormatAlpha | PixelFormatGDI | PixelFormatCanonical;

        let bitmap = GdipBitmap::new(
            width,
            height,
            width * 4,
            PixelFormat32bppARGB,
            data.as_ptr(),
        )?;
        bitmap.create_hicon()
    }
}

impl TryFrom<DynamicImage> for Image {
    type Error = Error;

    fn try_from(value: DynamicImage) -> std::result::Result<Self, Self::Error> {
        Ok(Self(Rc::new(value)))
    }
}

impl TryFrom<&DynamicImage> for Image {
    type Error = Error;

    fn try_from(value: &DynamicImage) -> std::result::Result<Self, Self::Error> {
        Ok(Self(Rc::new(value.clone())))
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum IconSize {
    Small,
    Logical,
}

struct HwndIcon {
    image: Image,
    size: IconSize,
    msg: u32,
    _icon: WinIcon,
}

thread_local! {
    static HWND_ICONS: RefCell<BTreeMap<HWND, HwndIcon>> = const { RefCell::new(BTreeMap::new()) };
}

fn small_icon_size(dpi: u32) -> Size {
    let cx = unsafe { GetSystemMetricsForDpi(SM_CXSMICON, dpi) };
    let cy = unsafe { GetSystemMetricsForDpi(SM_CYSMICON, dpi) };
    Size::new(cx as _, cy as _)
}

fn target_size(hwnd: HWND, image: &Image, size: IconSize) -> (i32, i32) {
    let dpi = get_dpi_for_window(hwnd);
    let size = match size {
        IconSize::Small => small_icon_size(dpi),
        IconSize::Logical => Size::new(image.0.width() as _, image.0.height() as _).to_device(dpi),
    };
    (
        size.width.round().max(1.0) as _,
        size.height.round().max(1.0) as _,
    )
}

/// Create an icon for `hwnd` and send `msg` (`BM_SETIMAGE` or `STM_SETIMAGE`)
/// to set it, replacing any previous icon.
pub(crate) fn set_hwnd_icon(hwnd: HWND, image: &Image, size: IconSize, msg: u32) -> Result<()> {
    let icon = image.to_hicon(target_size(hwnd, image, size))?;
    unsafe { SendMessageW(hwnd, msg, IMAGE_ICON as _, icon.0 as _) };
    HWND_ICONS.with(|map| {
        map.borrow_mut().insert(
            hwnd,
            HwndIcon {
                image: image.clone(),
                size,
                msg,
                _icon: icon,
            },
        )
    });
    Ok(())
}

/// Clear the icon of `hwnd` on the control.
pub(crate) fn clear_hwnd_icon(hwnd: HWND) {
    let entry = HWND_ICONS.with(|map| map.borrow_mut().remove(&hwnd));
    if let Some(entry) = entry {
        unsafe { SendMessageW(hwnd, entry.msg, IMAGE_ICON as _, 0) };
    }
}

/// Recreate the icon of `hwnd` for its current DPI.
pub(crate) fn refresh_hwnd_icon(hwnd: HWND) -> Result<()> {
    let Some((image, size, msg)) = HWND_ICONS.with(|map| {
        map.borrow()
            .get(&hwnd)
            .map(|entry| (entry.image.clone(), entry.size, entry.msg))
    }) else {
        return Ok(());
    };
    let icon = image.to_hicon(target_size(hwnd, &image, size))?;
    unsafe { SendMessageW(hwnd, msg, IMAGE_ICON as _, icon.0 as _) };
    HWND_ICONS.with(|map| {
        map.borrow_mut().insert(
            hwnd,
            HwndIcon {
                image,
                size,
                msg,
                _icon: icon,
            },
        )
    });
    Ok(())
}

/// Destroy and forget the icon of `hwnd`.
pub(crate) fn remove_hwnd_icon(hwnd: HWND) {
    HWND_ICONS.with(|map| map.borrow_mut().remove(&hwnd));
}

/// The image of the icon of `hwnd`.
pub(crate) fn hwnd_icon_image(hwnd: HWND) -> Option<Image> {
    HWND_ICONS.with(|map| map.borrow().get(&hwnd).map(|entry| entry.image.clone()))
}

/// The logical size of the icon of `hwnd`.
pub(crate) fn hwnd_icon_size(hwnd: HWND) -> Option<Size> {
    HWND_ICONS.with(|map| {
        map.borrow().get(&hwnd).map(|entry| match entry.size {
            IconSize::Small => {
                let dpi = get_dpi_for_window(hwnd);
                small_icon_size(dpi).to_logical(dpi)
            }
            IconSize::Logical => Size::new(entry.image.0.width() as _, entry.image.0.height() as _),
        })
    })
}
