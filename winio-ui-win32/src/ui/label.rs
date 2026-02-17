use compio::driver::syscall;
use compio_log::{error, info};
use inherit_methods_macro::inherit_methods;
use windows_sys::{
    Win32::{
        Foundation::{HWND, LPARAM, LRESULT, WPARAM},
        System::SystemServices::{SS_CENTER, SS_LEFT, SS_NOTIFY, SS_RIGHT},
        UI::{
            Controls::WC_STATICW,
            HiDpi::GetDpiForWindow,
            Shell::{DefSubclassProc, SetWindowSubclass, ShellExecuteW},
            WindowsAndMessaging::{
                IDC_HAND, LoadCursorW, STN_CLICKED, SW_SHOW, SetCursor, WM_COMMAND, WM_SETCURSOR,
                WM_SETFONT, WS_CHILD, WS_EX_TRANSPARENT, WS_VISIBLE,
            },
        },
    },
    w,
};
use winio_handle::{AsContainer, AsWidget};
use winio_primitive::{HAlign, Point, Size};

use crate::{
    Error, Result, WindowMessageCommand,
    font::default_underline_font,
    ui::{Widget, with_u16c},
};

#[derive(Debug)]
pub struct Label {
    handle: Widget,
}

#[inherit_methods(from = "self.handle")]
impl Label {
    pub fn new(parent: impl AsContainer) -> Result<Self> {
        let handle = Widget::new(
            WC_STATICW,
            // Without SS_NOTIFY ToolTip won't work
            WS_CHILD | WS_VISIBLE | SS_LEFT | SS_NOTIFY,
            0,
            parent.as_container().as_win32(),
        )?;
        Ok(Self { handle })
    }

    pub fn is_visible(&self) -> Result<bool>;

    pub fn set_visible(&mut self, v: bool) -> Result<()>;

    pub fn is_enabled(&self) -> Result<bool>;

    pub fn set_enabled(&mut self, v: bool) -> Result<()>;

    pub fn preferred_size(&self) -> Result<Size> {
        self.handle.measure_text()
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

    pub fn text(&self) -> Result<String>;

    pub fn set_text(&mut self, s: impl AsRef<str>) -> Result<()>;

    pub fn halign(&self) -> Result<HAlign> {
        let style = self.handle.style()?;
        let style = if (style & SS_RIGHT) == SS_RIGHT {
            HAlign::Right
        } else if (style & SS_CENTER) == SS_CENTER {
            HAlign::Center
        } else {
            HAlign::Left
        };
        Ok(style)
    }

    pub fn set_halign(&mut self, align: HAlign) -> Result<()> {
        let mut style = self.handle.style()?;
        style &= !(SS_RIGHT);
        match align {
            HAlign::Center => style |= SS_CENTER,
            HAlign::Right => style |= SS_RIGHT,
            _ => style |= SS_LEFT,
        }
        self.handle.set_style(style)
    }

    pub fn is_transparent(&self) -> Result<bool> {
        Ok((self.handle.ex_style()? & WS_EX_TRANSPARENT) != 0)
    }

    pub fn set_transparent(&mut self, v: bool) -> Result<()> {
        let style = if v { WS_EX_TRANSPARENT } else { 0 };
        self.handle.set_ex_style(style)
    }
}

winio_handle::impl_as_widget!(Label, handle);

#[derive(Debug)]
pub struct LinkLabel {
    handle: Label,
    uri: String,
}

#[inherit_methods(from = "self.handle")]
impl LinkLabel {
    pub fn new(parent: impl AsContainer) -> Result<Self> {
        let handle = Label::new(parent)?;
        syscall!(
            BOOL,
            SetWindowSubclass(
                handle.handle.as_widget().as_win32(),
                Some(link_label_wnd_proc),
                0,
                0
            )
        )?;
        handle.handle.send_message(WM_SETFONT, 0, 0);
        Ok(Self {
            handle,
            uri: String::new(),
        })
    }

    pub fn is_visible(&self) -> Result<bool>;

    pub fn set_visible(&mut self, v: bool) -> Result<()>;

    pub fn is_enabled(&self) -> Result<bool>;

    pub fn set_enabled(&mut self, v: bool) -> Result<()>;

    pub fn preferred_size(&self) -> Result<Size>;

    pub fn loc(&self) -> Result<Point>;

    pub fn set_loc(&mut self, p: Point) -> Result<()>;

    pub fn size(&self) -> Result<Size>;

    pub fn set_size(&mut self, v: Size) -> Result<()>;

    pub fn tooltip(&self) -> Result<String>;

    pub fn set_tooltip(&mut self, s: impl AsRef<str>) -> Result<()>;

    pub fn text(&self) -> Result<String>;

    pub fn set_text(&mut self, s: impl AsRef<str>) -> Result<()>;

    pub fn uri(&self) -> Result<String> {
        Ok(self.uri.clone())
    }

    pub fn set_uri(&mut self, s: impl AsRef<str>) -> Result<()> {
        self.uri = s.as_ref().to_string();
        Ok(())
    }

    pub async fn wait_click(&self) {
        loop {
            let WindowMessageCommand {
                message, handle, ..
            } = self.handle.handle.wait_parent(WM_COMMAND).await.command();
            if std::ptr::eq(handle, self.handle.as_widget().as_win32()) && message == STN_CLICKED {
                if !self.uri.is_empty() {
                    info!("Opening link: {}", self.uri);
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

pub(crate) unsafe extern "system" fn link_label_wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
    _id: usize,
    _data: usize,
) -> LRESULT {
    match msg {
        WM_SETCURSOR => unsafe {
            SetCursor(LoadCursorW(std::ptr::null_mut(), IDC_HAND));
            return 1;
        },
        WM_SETFONT => unsafe {
            match default_underline_font(GetDpiForWindow(hwnd)) {
                Ok(font) => {
                    return DefSubclassProc(hwnd, WM_SETFONT, font as _, lparam);
                }
                Err(_e) => {
                    error!("Failed to set underline font: {}", _e);
                }
            }
        },
        _ => {}
    }
    unsafe { DefSubclassProc(hwnd, msg, wparam, lparam) }
}
