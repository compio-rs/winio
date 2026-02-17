use compio_log::error;
use inherit_methods_macro::inherit_methods;
use windows_sys::{
    Win32::UI::{
        Controls::{LWS_TRANSPARENT, NM_CLICK, NM_RETURN, WC_LINK},
        Shell::ShellExecuteW,
        WindowsAndMessaging::{SW_SHOW, WM_NOTIFY, WS_CHILD, WS_TABSTOP, WS_VISIBLE},
    },
    w,
};
use winio_handle::{AsContainer, AsWidget};
use winio_primitive::{Point, Size};

use crate::{
    Error, Result, WindowMessageNotify,
    ui::{Widget, with_u16c},
};

#[derive(Debug)]
pub struct LinkLabel {
    handle: Widget,
    text: String,
    uri: String,
}

#[inherit_methods(from = "self.handle")]
impl LinkLabel {
    pub fn new(parent: impl AsContainer) -> Result<Self> {
        let handle = Widget::new(
            WC_LINK,
            WS_TABSTOP | WS_VISIBLE | WS_CHILD | LWS_TRANSPARENT,
            0,
            parent.as_container().as_win32(),
        )?;
        Ok(Self {
            handle,
            text: String::new(),
            uri: String::new(),
        })
    }

    pub fn is_visible(&self) -> Result<bool>;

    pub fn set_visible(&mut self, v: bool) -> Result<()>;

    pub fn is_enabled(&self) -> Result<bool>;

    pub fn set_enabled(&mut self, v: bool) -> Result<()>;

    pub fn preferred_size(&self) -> Result<Size> {
        with_u16c(&self.text, |text| self.handle.measure(text.as_ustr()))
    }

    pub fn loc(&self) -> Result<Point>;

    pub fn set_loc(&mut self, p: Point) -> Result<()>;

    pub fn size(&self) -> Result<Size>;

    pub fn set_size(&mut self, v: Size) -> Result<()>;

    pub fn tooltip(&self) -> Result<String>;

    pub fn set_tooltip(&mut self, s: impl AsRef<str>) -> Result<()>;

    fn refresh_text(&mut self) -> Result<()> {
        let s = if self.uri.is_empty() {
            format!(r#"<A ID="custom">{}</A>"#, self.text)
        } else {
            format!(r#"<A HREF="{}">{}</A>"#, self.uri, self.text)
        };
        self.handle.set_text(s)
    }

    pub fn text(&self) -> Result<String> {
        Ok(self.text.clone())
    }

    pub fn set_text(&mut self, s: impl AsRef<str>) -> Result<()> {
        self.text = s.as_ref().to_string();
        self.refresh_text()
    }

    pub fn uri(&self) -> Result<String> {
        Ok(self.uri.clone())
    }

    pub fn set_uri(&mut self, s: impl AsRef<str>) -> Result<()> {
        self.uri = s.as_ref().to_string();
        self.refresh_text()
    }

    pub async fn wait_click(&self) {
        loop {
            let WindowMessageNotify {
                hwnd_from, code, ..
            } = self.handle.wait_parent(WM_NOTIFY).await.notify();
            if std::ptr::eq(hwnd_from, self.handle.as_widget().as_win32())
                && matches!(code, NM_CLICK | NM_RETURN)
            {
                if !self.uri.is_empty() {
                    unsafe {
                        if let Err(_e) = with_u16c(&self.uri, |uri| {
                            let res = ShellExecuteW(
                                std::ptr::null_mut(),
                                w!("open"),
                                uri.as_ptr(),
                                std::ptr::null(),
                                std::ptr::null(),
                                SW_SHOW,
                            );
                            if res as usize <= 32 {
                                Err(Error::from_thread())
                            } else {
                                Ok(())
                            }
                        }) {
                            error!("Failed to open link: {}", _e);
                        }
                    }
                } else {
                    break;
                }
            }
        }
    }
}

winio_handle::impl_as_widget!(LinkLabel, handle);
