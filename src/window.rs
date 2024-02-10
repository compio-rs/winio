use std::{io, ptr::null, sync::OnceLock};

use widestring::U16CString;
use windows_sys::{
    w,
    Win32::{
        Foundation::HWND,
        System::LibraryLoader::GetModuleHandleW,
        UI::WindowsAndMessaging::{
            CloseWindow, CreateWindowExW, DestroyWindow, GetWindowTextLengthW, GetWindowTextW,
            LoadCursorW, RegisterClassExW, SetWindowTextW, ShowWindow, CW_USEDEFAULT, IDC_ARROW,
            SW_SHOWNORMAL, WM_CLOSE, WM_CREATE, WM_DESTROY, WNDCLASSEXW, WS_OVERLAPPEDWINDOW,
        },
    },
};

use crate::{syscall_bool, wait};

pub trait AsRawWindow {
    fn as_raw_window(&self) -> HWND;
}

pub trait IntoRawWindow {
    fn into_raw_window(self) -> HWND;
}

pub trait FromRawWindow {
    /// # Safety
    /// Caller should ensure the handle being valid.
    unsafe fn from_raw_window(handle: HWND) -> Self;
}

pub struct OwnedWindow(HWND);

impl Drop for OwnedWindow {
    fn drop(&mut self) {
        unsafe { CloseWindow(self.0) };
    }
}

impl AsRawWindow for OwnedWindow {
    fn as_raw_window(&self) -> HWND {
        self.0
    }
}

impl IntoRawWindow for OwnedWindow {
    fn into_raw_window(self) -> HWND {
        self.0
    }
}

impl FromRawWindow for OwnedWindow {
    unsafe fn from_raw_window(handle: HWND) -> Self {
        Self(handle)
    }
}

pub(crate) struct Widget(OwnedWindow);

impl Widget {
    pub fn new(
        class_name: *const u16,
        style: u32,
        ex_style: u32,
        parent: HWND,
    ) -> io::Result<Self> {
        let handle = unsafe {
            CreateWindowExW(
                ex_style,
                class_name,
                null(),
                style,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                parent,
                0,
                GetModuleHandleW(null()),
                null(),
            )
        };
        if handle != 0 {
            Ok(Self(unsafe { OwnedWindow::from_raw_window(handle) }))
        } else {
            Err(io::Error::last_os_error())
        }
    }

    pub fn text(&self) -> io::Result<String> {
        let handle = self.as_raw_window();
        let len = unsafe { GetWindowTextLengthW(handle) };
        if len == 0 {
            return Ok(String::new());
        };
        let mut res: Vec<u16> = Vec::with_capacity(len as usize + 1);
        syscall_bool(unsafe { GetWindowTextW(handle, res.as_mut_ptr(), res.capacity() as _) })?;
        unsafe { res.set_len(len as usize + 1) };
        Ok(unsafe { U16CString::from_vec_unchecked(res) }.to_string_lossy())
    }

    pub fn set_text(&self, s: impl AsRef<str>) -> io::Result<()> {
        let handle = self.as_raw_window();
        let s = U16CString::from_str_truncate(s);
        syscall_bool(unsafe { SetWindowTextW(handle, s.as_ptr()) })?;
        Ok(())
    }
}

impl AsRawWindow for Widget {
    fn as_raw_window(&self) -> HWND {
        self.0.as_raw_window()
    }
}

pub const WINDOW_CLASS_NAME: *const u16 = w!("XamlWindow");

fn register() -> io::Result<()> {
    let cls = WNDCLASSEXW {
        cbSize: std::mem::size_of::<WNDCLASSEXW>() as _,
        style: 0,
        lpfnWndProc: Some(crate::window_proc),
        cbClsExtra: 0,
        cbWndExtra: 0,
        hInstance: unsafe { GetModuleHandleW(null()) },
        hIcon: 0,
        hCursor: unsafe { LoadCursorW(0, IDC_ARROW) },
        hbrBackground: 0,
        lpszMenuName: null(),
        lpszClassName: WINDOW_CLASS_NAME,
        hIconSm: 0,
    };
    let res = unsafe { RegisterClassExW(&cls) };
    if res != 0 {
        Ok(())
    } else {
        Err(io::Error::last_os_error())
    }
}

static REGISTER: OnceLock<()> = OnceLock::new();

fn register_once() -> io::Result<()> {
    REGISTER.get_or_try_init(register)?;
    Ok(())
}

pub struct Window {
    handle: Widget,
}

impl Window {
    pub async fn new() -> io::Result<Self> {
        register_once()?;
        let this = Self {
            handle: Widget::new(WINDOW_CLASS_NAME, WS_OVERLAPPEDWINDOW, 0, 0)?,
        };
        unsafe { wait(this.as_raw_window(), WM_CREATE) }.await;
        unsafe { ShowWindow(this.as_raw_window(), SW_SHOWNORMAL) };
        Ok(this)
    }

    pub fn text(&self) -> io::Result<String> {
        self.handle.text()
    }

    pub fn set_text(&self, s: impl AsRef<str>) -> io::Result<()> {
        self.handle.set_text(s)
    }

    pub async fn close(&self) {
        unsafe { wait(self.as_raw_window(), WM_CLOSE) }.await;
    }

    pub async fn destory(&self) -> io::Result<()> {
        let fut = unsafe { wait(self.as_raw_window(), WM_DESTROY) };
        syscall_bool(unsafe { DestroyWindow(self.as_raw_window()) })?;
        fut.await;
        Ok(())
    }
}

impl AsRawWindow for Window {
    fn as_raw_window(&self) -> HWND {
        self.handle.as_raw_window()
    }
}
