use inherit_methods_macro::inherit_methods;
use windows_sys::Win32::{
    System::SystemServices::{SS_ICON, SS_NOTIFY, SS_REALSIZEIMAGE},
    UI::{
        Controls::WC_STATICW,
        WindowsAndMessaging::{STM_SETIMAGE, WS_CHILD, WS_VISIBLE},
    },
};
use winio_handle::{AsContainer, AsWidget};
use winio_primitive::{Point, Size};

use crate::{
    Image, Result,
    platform::image::{IconSize, clear_hwnd_icon, hwnd_icon_size, remove_hwnd_icon, set_hwnd_icon},
    widgets::Widget,
};

#[derive(Debug)]
pub struct Picture {
    handle: Widget,
}

#[inherit_methods(from = "self.handle")]
impl Picture {
    pub fn new(parent: impl AsContainer) -> Result<Self> {
        let handle = Widget::new(
            WC_STATICW,
            WS_CHILD | WS_VISIBLE | SS_ICON | SS_REALSIZEIMAGE | SS_NOTIFY,
            0,
            parent.as_container().as_win32(),
        )?;
        Ok(Self { handle })
    }

    pub fn is_visible(&self) -> Result<bool>;

    pub fn set_visible(&mut self, v: bool) -> Result<()>;

    pub fn preferred_size(&self) -> Result<Size> {
        Ok(hwnd_icon_size(self.handle.as_widget().as_win32()).unwrap_or_default())
    }

    pub fn loc(&self) -> Result<Point>;

    pub fn set_loc(&mut self, p: Point) -> Result<()>;

    pub fn size(&self) -> Result<Size>;

    pub fn set_size(&mut self, v: Size) -> Result<()> {
        self.handle.set_size(v)?;
        self.handle.invalidate(true)
    }

    pub fn tooltip(&self) -> Result<String>;

    pub fn set_tooltip(&mut self, s: impl AsRef<str>) -> Result<()>;

    pub fn set_image(&mut self, image: Option<&Image>) -> Result<()> {
        let hwnd = self.handle.as_widget().as_win32();
        match image {
            Some(image) => {
                set_hwnd_icon(hwnd, image, IconSize::Logical, STM_SETIMAGE)?;
            }
            None => clear_hwnd_icon(hwnd),
        }
        self.handle.invalidate(true)
    }
}

winio_handle::impl_as_widget!(Picture, handle);

impl Drop for Picture {
    fn drop(&mut self) {
        remove_hwnd_icon(self.handle.as_widget().as_win32());
    }
}
