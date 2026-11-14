use std::sync::OnceLock;

use inherit_methods_macro::inherit_methods;
use windows_sys::Win32::{
    Foundation::SIZE,
    UI::{
        Controls::{
            BCM_GETIDEALSIZE, BP_PUSHBUTTON, BS_SPLITBUTTON, PBS_NORMAL, TMT_CONTENTMARGINS,
            VSCLASS_BUTTON, WC_BUTTONW,
        },
        WindowsAndMessaging::{
            BM_SETIMAGE, BN_CLICKED, BS_ICON, BS_PUSHBUTTON, GetParent, WM_COMMAND, WS_CHILD,
            WS_TABSTOP, WS_VISIBLE,
        },
    },
};
use winio_handle::{AsContainer, AsWidget};
use winio_primitive::{Margin, Point, Size};

use crate::{
    Image, Result,
    platform::{
        image::{IconSize, clear_hwnd_icon, hwnd_icon_image, hwnd_icon_size, set_hwnd_icon},
        theme::Theme,
    },
    runtime::{WindowMessage, WindowMessageCommand},
    widgets::{Widget, edge},
};

#[derive(Debug)]
pub struct Button {
    handle: Widget,
    bs_icon: bool,
    split: bool,
}

const fn button_style(icon: bool, split: bool) -> u32 {
    WS_TABSTOP
        | WS_VISIBLE
        | WS_CHILD
        | (if split {
            BS_SPLITBUTTON as u32
        } else {
            BS_PUSHBUTTON as u32
        })
        | (if icon { BS_ICON as u32 } else { 0 })
}

#[inherit_methods(from = "self.handle")]
impl Button {
    pub fn new(parent: impl AsContainer) -> Result<Self> {
        Self::new_impl(parent, false)
    }

    pub(crate) fn new_split(parent: impl AsContainer) -> Result<Self> {
        Self::new_impl(parent, true)
    }

    fn new_impl(parent: impl AsContainer, split: bool) -> Result<Self> {
        let handle = Widget::new(
            WC_BUTTONW,
            button_style(false, split),
            0,
            parent.as_container().as_win32(),
        )?;
        Ok(Self {
            handle,
            bs_icon: false,
            split,
        })
    }

    pub fn is_visible(&self) -> Result<bool>;

    pub fn set_visible(&mut self, v: bool) -> Result<()>;

    pub fn is_enabled(&self) -> Result<bool>;

    pub fn set_enabled(&mut self, v: bool) -> Result<()>;

    pub fn preferred_size(&self) -> Result<Size> {
        let hwnd = self.handle.as_widget().as_win32();
        // The button itself knows its ideal size: it accounts for the text, the
        // icon, the drop-down arrow of a split button and the padding of the
        // current visual style.
        let mut size = SIZE { cx: 0, cy: 0 };
        if self
            .handle
            .send_message(BCM_GETIDEALSIZE, 0, &raw mut size as _)
            != 0
            && size.cx > 0
            && size.cy > 0
        {
            return Ok(self.handle.size_d2l((size.cx, size.cy)));
        }

        let mut s = self.handle.measure_text()?;
        if let Some(icon) = hwnd_icon_size(hwnd) {
            s.width += icon.width as f64;
            s.height = s.height.max(icon.height as f64);
        }
        let padding = self.content_padding();
        Ok(Size::new(
            s.width + padding.left + padding.right,
            s.height + padding.top + padding.bottom,
        ))
    }

    pub fn min_size(&self) -> Result<Size> {
        Ok(Size::zero())
    }

    fn content_padding(&self) -> Margin {
        static PADDING: OnceLock<Margin> = OnceLock::new();
        *PADDING.get_or_init(|| {
            let hwnd = self.handle.as_widget().as_win32();
            let edge = edge();
            Theme::open(hwnd, VSCLASS_BUTTON)
                .and_then(|theme| theme.margins(BP_PUSHBUTTON, PBS_NORMAL, TMT_CONTENTMARGINS))
                .unwrap_or_else(|| Margin::new_all_same(edge))
        })
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
        let new_handle = Widget::new(WC_BUTTONW, button_style(icon, self.split), 0, parent)?;
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
        clear_hwnd_icon(old_hwnd);
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

    pub(crate) async fn wait_parent(&self, msg: u32) -> WindowMessage {
        self.handle.wait_parent(msg).await
    }
}

winio_handle::impl_as_widget!(Button, handle);

impl Drop for Button {
    fn drop(&mut self) {
        clear_hwnd_icon(self.handle.as_widget().as_win32());
    }
}
