use std::{
    cell::Cell,
    ptr::{null, null_mut},
    sync::OnceLock,
};

use widestring::U16CString;
use windows_core::{Error, Result};
use windows_sys::Win32::{
    Foundation::{ERROR_CLASS_ALREADY_EXISTS, HWND, LPARAM, LRESULT, WPARAM},
    System::{
        DataExchange::{
            AddClipboardFormatListener, CloseClipboard, EmptyClipboard, EnumClipboardFormats,
            GetClipboardSequenceNumber, OpenClipboard, RemoveClipboardFormatListener,
        },
        LibraryLoader::GetModuleHandleW,
    },
    UI::WindowsAndMessaging::{
        CreateWindowExW, DefWindowProcW, DestroyWindow, GWLP_USERDATA, GetWindowLongPtrW,
        HWND_MESSAGE, RegisterClassW, SetWindowLongPtrW, WM_CLIPBOARDUPDATE, WNDCLASSW,
    },
};
use winio_callback::Callback;
use winio_pollable::GlobalRuntime;

use crate::{
    Data,
    platform::format::{copy_clipboard_hglobal, write_clipboard},
};

pub struct Clipboard {
    hwnd: HWND,
    inner: Box<ClipboardInner>,
}

struct ClipboardInner {
    on_changed: Callback<()>,
    writing: Cell<bool>,
    own_sequence: Cell<u32>,
}

impl Clipboard {
    pub fn new() -> Result<Self> {
        ensure_class()?;
        let class = class_name();
        let hwnd = unsafe {
            CreateWindowExW(
                0,
                class.as_ptr(),
                null(),
                0,
                0,
                0,
                0,
                0,
                HWND_MESSAGE,
                null_mut(),
                GetModuleHandleW(null()),
                null(),
            )
        };
        if hwnd.is_null() {
            return Err(Error::from_thread());
        }
        let inner = Box::new(ClipboardInner {
            on_changed: Callback::new(),
            writing: Cell::new(false),
            own_sequence: Cell::new(0),
        });
        unsafe {
            SetWindowLongPtrW(hwnd, GWLP_USERDATA, &*inner as *const _ as isize);
        }
        if unsafe { AddClipboardFormatListener(hwnd) } == 0 {
            unsafe { DestroyWindow(hwnd) };
            return Err(Error::from_thread());
        }
        Ok(Self { hwnd, inner })
    }

    pub async fn read(&mut self) -> Result<Option<Data>> {
        let _guard = ClipboardGuard::open(self.hwnd)?;
        let mut data = Data::new()?;
        let mut format = 0;
        loop {
            format = unsafe { EnumClipboardFormats(format) };
            if format == 0 {
                break;
            }
            if let Some(hglobal) = copy_clipboard_hglobal(format)? {
                data.insert_hglobal(format, hglobal);
            }
        }
        if data.is_empty() {
            Ok(None)
        } else {
            Ok(Some(data))
        }
    }

    pub async fn write(&mut self, data: &Data) -> Result<()> {
        let hwnd = self.hwnd;
        self.with_write(|| {
            let _guard = ClipboardGuard::open(hwnd)?;
            if unsafe { EmptyClipboard() } == 0 {
                return Err(Error::from_thread());
            }
            write_clipboard(data)
        })
    }

    pub fn clear(&mut self) -> Result<()> {
        let hwnd = self.hwnd;
        self.with_write(|| {
            let _guard = ClipboardGuard::open(hwnd)?;
            if unsafe { EmptyClipboard() } == 0 {
                return Err(Error::from_thread());
            }
            Ok(())
        })
    }

    pub async fn wait_change(&self) {
        self.inner.on_changed.wait().await
    }

    fn with_write<T>(&mut self, f: impl FnOnce() -> Result<T>) -> Result<T> {
        self.inner.writing.set(true);
        let _guard = WriteGuard(&self.inner.writing);
        let result = f();
        if result.is_ok() {
            self.inner
                .own_sequence
                .set(unsafe { GetClipboardSequenceNumber() });
        }
        result
    }
}

impl Drop for Clipboard {
    fn drop(&mut self) {
        unsafe {
            RemoveClipboardFormatListener(self.hwnd);
            DestroyWindow(self.hwnd);
        }
    }
}

impl std::fmt::Debug for Clipboard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Clipboard").finish_non_exhaustive()
    }
}

impl ClipboardInner {
    fn notify(&self) {
        if self.writing.get() {
            return;
        }
        let sequence = unsafe { GetClipboardSequenceNumber() };
        if sequence != 0 && sequence == self.own_sequence.get() {
            return;
        }
        self.on_changed.signal::<GlobalRuntime>(());
    }
}

struct WriteGuard<'a>(&'a Cell<bool>);

impl Drop for WriteGuard<'_> {
    fn drop(&mut self) {
        self.0.set(false);
    }
}

struct ClipboardGuard;

impl ClipboardGuard {
    fn open(hwnd: HWND) -> Result<Self> {
        if unsafe { OpenClipboard(hwnd) } == 0 {
            Err(Error::from_thread())
        } else {
            Ok(Self)
        }
    }
}

impl Drop for ClipboardGuard {
    fn drop(&mut self) {
        unsafe { CloseClipboard() };
    }
}

fn class_name() -> U16CString {
    U16CString::from_str("WinioClipboardListener").expect("valid static string")
}

static CLASS: OnceLock<std::result::Result<(), u32>> = OnceLock::new();

fn ensure_class() -> Result<()> {
    let result = CLASS.get_or_init(|| {
        let name = class_name();
        let class = WNDCLASSW {
            style: 0,
            lpfnWndProc: Some(wndproc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: unsafe { GetModuleHandleW(null()) },
            hIcon: null_mut(),
            hCursor: null_mut(),
            hbrBackground: null_mut(),
            lpszMenuName: null(),
            lpszClassName: name.as_ptr(),
        };
        if unsafe { RegisterClassW(&class) } == 0 {
            let code = unsafe { windows_sys::Win32::Foundation::GetLastError() };
            if code != ERROR_CLASS_ALREADY_EXISTS {
                return Err(code);
            }
        }
        Ok(())
    });
    result.map_err(|_| Error::from_thread())
}

unsafe extern "system" fn wndproc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if msg == WM_CLIPBOARDUPDATE {
        let inner = unsafe { GetWindowLongPtrW(hwnd, GWLP_USERDATA) } as *const ClipboardInner;
        if let Some(inner) = unsafe { inner.as_ref() } {
            inner.notify();
        }
        return 0;
    }
    unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
}
