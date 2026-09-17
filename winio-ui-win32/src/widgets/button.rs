use inherit_methods_macro::inherit_methods;
use windows_sys::Win32::UI::{
    Controls::WC_BUTTONW,
    WindowsAndMessaging::{
        BM_SETIMAGE, BN_CLICKED, BS_ICON, BS_PUSHBUTTON, GetParent, WM_COMMAND, WS_CHILD,
        WS_TABSTOP, WS_VISIBLE,
    },
};
use winio_handle::{AsContainer, AsWidget};
use winio_primitive::{Point, Size};

use crate::{
    Image, Result,
    platform::image::{
        IconSize, clear_hwnd_icon, hwnd_icon_image, hwnd_icon_size, remove_hwnd_icon, set_hwnd_icon,
    },
    runtime::WindowMessageCommand,
    widgets::Widget,
};

#[derive(Debug)]
pub struct Button {
    handle: Widget,
    bs_icon: bool,
}

const fn button_style(icon: bool) -> u32 {
    WS_TABSTOP
        | WS_VISIBLE
        | WS_CHILD
        | BS_PUSHBUTTON as u32
        | if icon { BS_ICON as u32 } else { 0 }
}

#[inherit_methods(from = "self.handle")]
impl Button {
    pub fn new(parent: impl AsContainer) -> Result<Self> {
        let handle = Widget::new(
            WC_BUTTONW,
            button_style(false),
            0,
            parent.as_container().as_win32(),
        )?;
        Ok(Self {
            handle,
            bs_icon: false,
        })
    }

    pub fn is_visible(&self) -> Result<bool>;

    pub fn set_visible(&mut self, v: bool) -> Result<()>;

    pub fn is_enabled(&self) -> Result<bool>;

    pub fn set_enabled(&mut self, v: bool) -> Result<()>;

    pub fn preferred_size(&self) -> Result<Size> {
        let mut s = self.handle.measure_text()?;
        if let Some(icon) = hwnd_icon_size(self.handle.as_widget().as_win32()) {
            s.width += icon.width;
            s.height = s.height.max(icon.height);
        }
        Ok(Size::new(s.width + 4.0, s.height + 4.0))
    }

    pub fn loc(&self) -> Result<Point>;

    pub fn set_loc(&mut self, p: Point) -> Result<()>;

    pub fn size(&self) -> Result<Size>;

    pub fn set_size(&mut self, v: Size) -> Result<()>;

    pub fn tooltip(&self) -> Result<String>;

    pub fn set_tooltip(&mut self, s: impl AsRef<str>) -> Result<()>;

    pub fn text(&self) -> Result<String>;

    pub fn set_text(&mut self, s: impl AsRef<str>) -> Result<()> {
        self.handle.set_text(s)?;
        self.update_icon_style()
    }

    pub fn set_icon(&mut self, icon: Option<&Image>) -> Result<()> {
        let hwnd = self.handle.as_widget().as_win32();
        match icon {
            Some(image) => set_hwnd_icon(hwnd, image, IconSize::Small, BM_SETIMAGE)?,
            None => clear_hwnd_icon(hwnd),
        }
        self.update_icon_style()?;
        self.handle.invalidate(true)
    }

    /// Whether the button should have the `BS_ICON` style, i.e. the icon is
    /// shown on its own because there is no text.
    fn icon_style_needed(&self) -> Result<bool> {
        Ok(hwnd_icon_size(self.handle.as_widget().as_win32()).is_some()
            && self.handle.text()?.is_empty())
    }

    fn update_icon_style(&mut self) -> Result<()> {
        let icon = self.icon_style_needed()?;
        if self.bs_icon != icon {
            self.recreate(icon)?;
        }
        Ok(())
    }

    fn recreate(&mut self, icon: bool) -> Result<()> {
        let old_hwnd = self.handle.as_widget().as_win32();
        let image = hwnd_icon_image(old_hwnd);
        let parent = unsafe { GetParent(old_hwnd) };
        let mut new_handle = Widget::new(WC_BUTTONW, button_style(icon), 0, parent)?;
        new_handle.set_visible(self.handle.is_visible()?)?;
        new_handle.set_enabled(self.handle.is_enabled()?)?;
        new_handle.set_loc(self.handle.loc()?)?;
        new_handle.set_size(self.handle.size()?)?;
        new_handle.set_tooltip(self.handle.tooltip()?)?;
        new_handle.set_text(self.handle.text()?)?;
        if let Some(image) = image {
            set_hwnd_icon(
                new_handle.as_widget().as_win32(),
                &image,
                IconSize::Small,
                BM_SETIMAGE,
            )?;
        }
        remove_hwnd_icon(old_hwnd);
        self.handle = new_handle;
        self.bs_icon = icon;
        Ok(())
    }

    pub async fn wait_click(&self) {
        loop {
            let WindowMessageCommand {
                message, handle, ..
            } = self.handle.wait_parent(WM_COMMAND).await.command();
            if std::ptr::eq(handle, self.handle.as_widget().as_win32()) && (message == BN_CLICKED) {
                break;
            }
        }
    }
}

winio_handle::impl_as_widget!(Button, handle);

impl Drop for Button {
    fn drop(&mut self) {
        remove_hwnd_icon(self.handle.as_widget().as_win32());
    }
}
