use std::{io, mem::MaybeUninit, ptr::null, rc::Rc, sync::OnceLock};

use widestring::U16CString;
use windows_sys::{
    w,
    Win32::{
        Foundation::{HWND, POINT, RECT},
        Graphics::Gdi::{
            GetStockObject, MapWindowPoints, Rectangle, SelectObject, HDC, WHITE_BRUSH,
        },
        System::LibraryLoader::GetModuleHandleW,
        UI::WindowsAndMessaging::{
            CloseWindow, CreateWindowExW, GetClientRect, GetParent, GetWindowRect,
            GetWindowTextLengthW, GetWindowTextW, LoadCursorW, RegisterClassExW, SetWindowPos,
            SetWindowTextW, ShowWindow, CW_USEDEFAULT, HWND_DESKTOP, IDC_ARROW, MSG,
            SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE, SWP_NOZORDER, SW_SHOWNORMAL, WM_CLOSE,
            WM_DPICHANGED, WM_ERASEBKGND, WM_MOVE, WM_SIZE, WNDCLASSEXW, WS_OVERLAPPEDWINDOW,
        },
    },
};

use crate::{
    syscall_bool,
    ui::{
        dpi::{get_dpi_for_window, DpiAware},
        drawing::{Point, Size},
    },
    wait,
};

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

impl<T: AsRawWindow> AsRawWindow for &'_ T {
    fn as_raw_window(&self) -> HWND {
        (**self).as_raw_window()
    }
}

#[derive(Debug)]
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

#[derive(Debug)]
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

    pub async fn wait(&self, msg: u32) -> MSG {
        unsafe { wait(self.as_raw_window(), msg) }.await
    }

    pub async fn wait_parent(&self, msg: u32) -> MSG {
        unsafe { wait(GetParent(self.as_raw_window()), msg) }.await
    }

    pub fn dpi(&self) -> u32 {
        unsafe { get_dpi_for_window(self.as_raw_window()) }
    }

    pub fn size_d2l(&self, s: (i32, i32)) -> Size {
        let dpi = self.dpi();
        Size::new(s.0 as f64, s.1 as f64).to_logical(dpi)
    }

    pub fn size_l2d(&self, s: Size) -> (i32, i32) {
        let dpi = self.dpi();
        let s = s.to_device(dpi);
        (s.width as i32, s.height as i32)
    }

    pub fn point_d2l(&self, p: (i32, i32)) -> Point {
        let dpi = self.dpi();
        Point::new(p.0 as f64, p.1 as f64).to_logical(dpi)
    }

    pub fn point_l2d(&self, p: Point) -> (i32, i32) {
        let dpi = self.dpi();
        let p = p.to_device(dpi);
        (p.x as i32, p.y as i32)
    }

    fn sized(&self) -> io::Result<(i32, i32)> {
        let handle = self.as_raw_window();
        let mut rect = MaybeUninit::uninit();
        syscall_bool(unsafe { GetWindowRect(handle, rect.as_mut_ptr()) })?;
        let rect = unsafe { rect.assume_init() };
        Ok((rect.right - rect.left, rect.bottom - rect.top))
    }

    fn set_sized(&self, v: (i32, i32)) -> io::Result<()> {
        let handle = self.as_raw_window();
        if v != self.sized()? {
            syscall_bool(unsafe {
                SetWindowPos(handle, 0, 0, 0, v.0, v.1, SWP_NOMOVE | SWP_NOZORDER)
            })?;
        }
        Ok(())
    }

    pub fn size(&self) -> io::Result<Size> {
        Ok(self.size_d2l(self.sized()?))
    }

    pub fn set_size(&self, v: Size) -> io::Result<()> {
        self.set_sized(self.size_l2d(v))
    }

    fn locd(&self) -> io::Result<(i32, i32)> {
        let handle = self.as_raw_window();
        unsafe {
            let mut rect = MaybeUninit::uninit();
            syscall_bool(GetWindowRect(handle, rect.as_mut_ptr()))?;
            let rect = rect.assume_init();
            let mut point = POINT {
                x: rect.left,
                y: rect.top,
            };
            syscall_bool(MapWindowPoints(
                HWND_DESKTOP,
                GetParent(handle),
                &mut point,
                2,
            ))?;
            Ok((point.x, point.y))
        }
    }

    fn set_locd(&self, p: (i32, i32)) -> io::Result<()> {
        let handle = self.as_raw_window();
        if p != self.locd()? {
            syscall_bool(unsafe {
                SetWindowPos(handle, 0, p.0, p.1, 0, 0, SWP_NOSIZE | SWP_NOZORDER)
            })?;
        }
        Ok(())
    }

    pub fn loc(&self) -> io::Result<Point> {
        Ok(self.point_d2l(self.locd()?))
    }

    pub fn set_loc(&self, p: Point) -> io::Result<()> {
        self.set_locd(self.point_l2d(p))
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

#[derive(Debug, Clone)]
pub struct Window {
    handle: Rc<Widget>,
}

impl Window {
    pub fn new() -> io::Result<Self> {
        register_once()?;
        let handle = Widget::new(WINDOW_CLASS_NAME, WS_OVERLAPPEDWINDOW, 0, 0)?;
        let handle = Rc::<Widget>::new_cyclic(|weak_this| {
            crate::spawn({
                let weak_this = weak_this.clone();
                async move {
                    while let Some(this) = weak_this.upgrade() {
                        let msg = this.wait(WM_ERASEBKGND).await;
                        unsafe {
                            let hdc = msg.wParam as HDC;
                            let brush = GetStockObject(WHITE_BRUSH);
                            let old_brush = SelectObject(hdc, brush);
                            let mut r = MaybeUninit::uninit();
                            GetClientRect(this.as_raw_window(), r.as_mut_ptr());
                            let r = r.assume_init();
                            Rectangle(hdc, r.left - 1, r.top - 1, r.right + 1, r.bottom + 1);
                            SelectObject(hdc, old_brush);
                        }
                    }
                }
            })
            .detach();
            crate::spawn({
                let weak_this = weak_this.clone();
                async move {
                    while let Some(this) = weak_this.upgrade() {
                        let msg = this.wait(WM_DPICHANGED).await;
                        unsafe {
                            let new_rect = msg.lParam as *const RECT;
                            if let Some(new_rect) = new_rect.as_ref() {
                                SetWindowPos(
                                    this.as_raw_window(),
                                    0,
                                    new_rect.left,
                                    new_rect.top,
                                    new_rect.right - new_rect.left,
                                    new_rect.bottom - new_rect.top,
                                    SWP_NOZORDER | SWP_NOACTIVATE,
                                );
                            }
                        }
                    }
                }
            })
            .detach();
            handle
        });
        let this = Self { handle };
        unsafe { ShowWindow(this.as_raw_window(), SW_SHOWNORMAL) };
        Ok(this)
    }

    pub fn loc(&self) -> io::Result<Point> {
        self.handle.loc()
    }

    pub fn set_loc(&self, p: Point) -> io::Result<()> {
        self.handle.set_loc(p)
    }

    pub fn size(&self) -> io::Result<Size> {
        self.handle.size()
    }

    pub fn set_size(&self, v: Size) -> io::Result<()> {
        self.handle.set_size(v)
    }

    pub fn client_size(&self) -> io::Result<Size> {
        let handle = self.as_raw_window();
        let mut rect = MaybeUninit::uninit();
        syscall_bool(unsafe { GetClientRect(handle, rect.as_mut_ptr()) })?;
        let rect = unsafe { rect.assume_init() };
        Ok(self
            .handle
            .size_d2l((rect.right - rect.left, rect.bottom - rect.top)))
    }

    pub fn text(&self) -> io::Result<String> {
        self.handle.text()
    }

    pub fn set_text(&self, s: impl AsRef<str>) -> io::Result<()> {
        self.handle.set_text(s)
    }

    pub async fn wait_size(&self) {
        self.handle.wait(WM_SIZE).await;
    }

    pub async fn wait_move(&self) {
        self.handle.wait(WM_MOVE).await;
    }

    pub async fn wait_close(&self) {
        self.handle.wait(WM_CLOSE).await;
    }
}

impl AsRawWindow for Window {
    fn as_raw_window(&self) -> HWND {
        self.handle.as_raw_window()
    }
}
