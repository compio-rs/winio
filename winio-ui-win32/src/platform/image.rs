use std::{cell::RefCell, collections::BTreeMap, mem::zeroed, ptr::null_mut};

use image::{DynamicImage, imageops::FilterType};
use windows_sys::Win32::{
    Foundation::HWND,
    Graphics::GdiPlus::{
        GdipCreateBitmapFromScan0, GdipCreateHICONFromBitmap, GdipDisposeImage, GdiplusShutdown,
        GdiplusStartup, GdiplusStartupInput, GpBitmap,
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
use crate::Result;

struct WinIcon(HICON);

impl Drop for WinIcon {
    fn drop(&mut self) {
        unsafe { DestroyIcon(self.0) };
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
        let status = unsafe { GdiplusStartup(&mut token, &input, null_mut()) };
        assert_eq!(status, 0);
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

impl Drop for GdipBitmap {
    fn drop(&mut self) {
        unsafe {
            GdipDisposeImage(self.0.cast());
        }
    }
}

#[derive(Debug, Clone)]
pub struct Image(DynamicImage);

impl Image {
    pub fn new(image: DynamicImage) -> Result<Self> {
        Ok(Self(image))
    }

    pub fn size(&self) -> Result<Size> {
        Ok(Size::new(self.0.width() as _, self.0.height() as _))
    }

    pub fn set_size(&mut self, size: Size) -> Result<()> {
        let width = size.width.max(1.0).round() as u32;
        let height = size.height.max(1.0).round() as u32;
        self.0 = self.0.resize_exact(width, height, FilterType::Triangle);
        Ok(())
    }

    #[allow(non_upper_case_globals)]
    fn to_hicon(&self, (width, height): (i32, i32)) -> Result<WinIcon> {
        gdip_init()?;

        let image = self
            .0
            .resize_exact(width as _, height as _, FilterType::Triangle)
            .into_rgba8();

        const PixelFormatGDI: i32 = 0x00020000;
        const PixelFormatAlpha: i32 = 0x00040000;
        const PixelFormatCanonical: i32 = 0x00200000;
        const PixelFormat32bppARGB: i32 =
            10 | (32 << 8) | PixelFormatAlpha | PixelFormatGDI | PixelFormatCanonical;

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

        let mut bitmap = null_mut();
        let status = unsafe {
            GdipCreateBitmapFromScan0(
                width,
                height,
                width * 4,
                PixelFormat32bppARGB,
                data.as_ptr(),
                &mut bitmap,
            )
        };
        assert_eq!(status, 0);
        let bitmap = GdipBitmap(bitmap);
        let mut icon = null_mut();
        let status = unsafe { GdipCreateHICONFromBitmap(bitmap.0, &mut icon) };
        assert_eq!(status, 0);
        Ok(WinIcon(icon))
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum IconSize {
    Small,
    Logical(Size),
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

fn target_size(hwnd: HWND, size: IconSize) -> (i32, i32) {
    let dpi = get_dpi_for_window(hwnd);
    let size = match size {
        IconSize::Small => small_icon_size(dpi),
        IconSize::Logical(size) => size.to_device(dpi),
    };
    (
        size.width.round().max(1.0) as _,
        size.height.round().max(1.0) as _,
    )
}

/// Create an icon for `hwnd` and send `msg` (`BM_SETIMAGE` or `STM_SETIMAGE`)
/// to set it, replacing any previous icon.
pub(crate) fn set_hwnd_icon(hwnd: HWND, image: &Image, size: IconSize, msg: u32) -> Result<()> {
    let icon = image.to_hicon(target_size(hwnd, size))?;
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
    let icon = image.to_hicon(target_size(hwnd, size))?;
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
            IconSize::Logical(size) => size,
        })
    })
}
