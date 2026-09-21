use std::{
    borrow::Cow,
    cell::RefCell,
    collections::BTreeMap,
    ptr::{null, null_mut},
    rc::Rc,
};

use image::{DynamicImage, GenericImageView, imageops::FilterType};
use windows_core::Error;
use windows_sys::Win32::{
    Foundation::HWND,
    Graphics::Gdi::{
        BI_BITFIELDS, BITMAPV5HEADER, CreateBitmap, CreateDIBSection, DIB_RGB_COLORS, DeleteObject,
        GetDC, HBITMAP, HDC, ReleaseDC,
    },
    UI::{
        ColorSystem::LCS_WINDOWS_COLOR_SPACE,
        HiDpi::GetSystemMetricsForDpi,
        WindowsAndMessaging::{
            CreateIconIndirect, DestroyIcon, HICON, ICONINFO, IMAGE_ICON, SM_CXSMICON, SM_CYSMICON,
            SendMessageW,
        },
    },
};
use winio_primitive::BitmapSize;
use winio_ui_windows_common::rgba8_to_pbgra8;

use super::dpi::get_dpi_for_window;
use crate::{DrawingContext, DrawingImage, Result};

struct WinBitmap(HBITMAP);

impl WinBitmap {
    pub fn retain(h: HBITMAP) -> Result<Self> {
        if h.is_null() {
            Err(Error::from_thread())
        } else {
            Ok(Self(h))
        }
    }
}

impl Drop for WinBitmap {
    fn drop(&mut self) {
        unsafe { DeleteObject(self.0) };
    }
}

struct WinIcon(HICON);

impl WinIcon {
    pub fn retain(h: HICON) -> Result<Self> {
        if h.is_null() {
            Err(Error::from_thread())
        } else {
            Ok(Self(h))
        }
    }
}

impl Drop for WinIcon {
    fn drop(&mut self) {
        unsafe { DestroyIcon(self.0) };
    }
}

struct WinDC(HDC, HWND);

impl WinDC {
    pub fn new(hwnd: HWND) -> Result<Self> {
        let hdc = unsafe { GetDC(hwnd) };
        if hdc.is_null() {
            Err(Error::from_thread())
        } else {
            Ok(Self(hdc, hwnd))
        }
    }
}

impl Drop for WinDC {
    fn drop(&mut self) {
        unsafe { ReleaseDC(self.1, self.0) };
    }
}

#[derive(Debug, Clone)]
pub struct Image(Rc<DynamicImage>);

impl Image {
    pub fn size(&self) -> Result<BitmapSize> {
        Ok(BitmapSize::new(
            self.0.width() as usize,
            self.0.height() as usize,
        ))
    }

    pub fn try_to_drawing(&self, context: &DrawingContext) -> Result<DrawingImage> {
        context.create_image(Cow::Borrowed(&self.0))
    }

    #[allow(non_upper_case_globals)]
    fn to_hicon(&self, resize: Option<(i32, i32)>) -> Result<WinIcon> {
        let (mut data, width, height) = match resize {
            Some((width, height)) => {
                let data = self
                    .0
                    .resize_exact(width as _, height as _, FilterType::Triangle)
                    .into_rgba8()
                    .into_raw();
                (data, width, height)
            }
            None => {
                let (width, height) = self.0.dimensions();
                (self.0.to_rgba8().into_raw(), width as _, height as _)
            }
        };

        rgba8_to_pbgra8(&mut data);

        unsafe {
            let hdc = WinDC::new(null_mut())?;

            let mut header: BITMAPV5HEADER = std::mem::zeroed();
            header.bV5Size = std::mem::size_of::<BITMAPV5HEADER>() as _;
            header.bV5Width = width;
            header.bV5Height = -height;
            header.bV5Planes = 1;
            header.bV5BitCount = 32;
            header.bV5Compression = BI_BITFIELDS;
            header.bV5SizeImage = (width * height * 4) as _;
            header.bV5RedMask = 0x00FF0000;
            header.bV5GreenMask = 0x0000FF00;
            header.bV5BlueMask = 0x000000FF;
            header.bV5AlphaMask = 0xFF000000;
            header.bV5CSType = LCS_WINDOWS_COLOR_SPACE as _;
            let mut bits = null_mut();

            let bitmap = WinBitmap::retain(CreateDIBSection(
                hdc.0,
                &raw const header as _,
                DIB_RGB_COLORS,
                &mut bits,
                null_mut(),
                0,
            ))?;

            let slice =
                std::slice::from_raw_parts_mut(bits.cast::<u8>(), (width * height * 4) as _);
            slice.copy_from_slice(&data);

            let mask = WinBitmap::retain(CreateBitmap(width, height, 1, 0, null()))?;

            let mut info: ICONINFO = std::mem::zeroed();
            info.fIcon = 1;
            info.hbmColor = bitmap.0;
            info.hbmMask = mask.0;

            WinIcon::retain(CreateIconIndirect(&info))
        }
    }
}

impl TryFrom<DynamicImage> for Image {
    type Error = Error;

    fn try_from(value: DynamicImage) -> Result<Self> {
        Ok(Self(Rc::new(value)))
    }
}

impl TryFrom<&DynamicImage> for Image {
    type Error = Error;

    fn try_from(value: &DynamicImage) -> Result<Self> {
        Ok(Self(Rc::new(value.clone())))
    }
}

impl TryFrom<&DrawingImage> for Image {
    type Error = Error;

    fn try_from(value: &DrawingImage) -> Result<Self> {
        Ok(Self(Rc::new(value.to_dynamic_image()?)))
    }
}

impl TryFrom<DrawingImage> for Image {
    type Error = Error;

    fn try_from(value: DrawingImage) -> Result<Self> {
        Self::try_from(&value)
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

fn small_icon_size(dpi: u32) -> (i32, i32) {
    let cx = unsafe { GetSystemMetricsForDpi(SM_CXSMICON, dpi) };
    let cy = unsafe { GetSystemMetricsForDpi(SM_CYSMICON, dpi) };
    (cx, cy)
}

fn target_size(hwnd: HWND, size: IconSize) -> Option<(i32, i32)> {
    let dpi = get_dpi_for_window(hwnd);
    match size {
        IconSize::Small => Some(small_icon_size(dpi)),
        IconSize::Logical => None,
    }
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

/// The image of the icon of `hwnd`.
pub(crate) fn hwnd_icon_image(hwnd: HWND) -> Option<Image> {
    HWND_ICONS.with(|map| map.borrow().get(&hwnd).map(|entry| entry.image.clone()))
}

/// The logical size of the icon of `hwnd`.
pub(crate) fn hwnd_icon_size(hwnd: HWND) -> Option<BitmapSize> {
    HWND_ICONS.with(|map| {
        map.borrow().get(&hwnd).map(|entry| match entry.size {
            IconSize::Small => {
                let (width, height) = small_icon_size(96);
                BitmapSize::new(width as _, height as _)
            }
            IconSize::Logical => {
                BitmapSize::new(entry.image.0.width() as _, entry.image.0.height() as _)
            }
        })
    })
}
