use std::{
    mem::MaybeUninit,
    ptr::{null, null_mut},
    sync::OnceLock,
};

use compio::driver::syscall;
use widestring::U16CString;
use windows_sys::{
    Win32::{
        Foundation::{HWND, POINT, SetLastError},
        Graphics::Gdi::MapWindowPoints,
        System::LibraryLoader::GetModuleHandleW,
        UI::{
            Input::KeyboardAndMouse::{EnableWindow, IsWindowEnabled},
            WindowsAndMessaging::{
                CW_USEDEFAULT, CloseWindow, CreateWindowExW, DestroyWindow, GWL_STYLE,
                GetClientRect, GetParent, GetWindowLongPtrW, GetWindowRect, GetWindowTextLengthW,
                GetWindowTextW, HICON, HWND_DESKTOP, ICON_BIG, IDC_ARROW, IMAGE_ICON,
                IsWindowVisible, LR_DEFAULTCOLOR, LR_DEFAULTSIZE, LR_SHARED, LoadCursorW,
                LoadImageW, RegisterClassExW, SW_HIDE, SW_SHOW, SWP_NOMOVE, SWP_NOSIZE,
                SWP_NOZORDER, SendMessageW, SetWindowLongPtrW, SetWindowPos, SetWindowTextW,
                ShowWindow, WM_CLOSE, WM_MOVE, WM_SETICON, WM_SIZE, WNDCLASSEXW, WS_CHILDWINDOW,
                WS_OVERLAPPEDWINDOW,
            },
        },
    },
    w,
};

use crate::{
    AsRawWindow, AsWindow, Point, Size,
    runtime::{WindowMessage, wait, window_proc},
    ui::{
        RawWindow,
        darkmode::{
            PreferredAppMode, control_use_dark_mode, set_preferred_app_mode, window_use_dark_mode,
        },
        dpi::{DpiAware, get_dpi_for_window},
    },
};

#[derive(Debug)]
pub struct OwnedWindow(HWND);

impl OwnedWindow {
    pub unsafe fn from_raw_window(h: HWND) -> Self {
        Self(h)
    }

    pub fn as_raw_window(&self) -> HWND {
        self.0
    }
}

impl Drop for OwnedWindow {
    fn drop(&mut self) {
        unsafe { CloseWindow(self.0) };
    }
}

impl AsRawWindow for OwnedWindow {
    fn as_raw_window(&self) -> RawWindow {
        self.0
    }
}

#[derive(Debug)]
pub(crate) struct Widget(OwnedWindow);

impl Widget {
    pub fn new(class_name: *const u16, style: u32, ex_style: u32, parent: HWND) -> Self {
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
                null_mut(),
                GetModuleHandleW(null()),
                null(),
            )
        };
        if handle.is_null() {
            panic!("{:?}", std::io::Error::last_os_error());
        }
        unsafe {
            control_use_dark_mode(handle, false);
            crate::runtime::refresh_font(handle);
            Self(OwnedWindow::from_raw_window(handle))
        }
    }

    pub fn as_raw_window(&self) -> HWND {
        self.0.as_raw_window()
    }

    pub async fn wait(&self, msg: u32) -> WindowMessage {
        unsafe { wait(self.as_raw_window(), msg) }.await
    }

    pub async fn wait_parent(&self, msg: u32) -> WindowMessage {
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

    fn sized(&self) -> (i32, i32) {
        let handle = self.as_raw_window();
        let mut rect = MaybeUninit::uninit();
        syscall!(BOOL, unsafe { GetWindowRect(handle, rect.as_mut_ptr()) }).unwrap();
        let rect = unsafe { rect.assume_init() };
        (rect.right - rect.left, rect.bottom - rect.top)
    }

    fn set_sized(&mut self, v: (i32, i32)) {
        let handle = self.as_raw_window();
        if v != self.sized() {
            syscall!(
                BOOL,
                SetWindowPos(
                    handle,
                    null_mut(),
                    0,
                    0,
                    v.0,
                    v.1,
                    SWP_NOMOVE | SWP_NOZORDER,
                )
            )
            .unwrap();
        }
    }

    pub fn size(&self) -> Size {
        self.size_d2l(self.sized())
    }

    pub fn set_size(&mut self, v: Size) {
        self.set_sized(self.size_l2d(v))
    }

    fn locd(&self) -> (i32, i32) {
        let handle = self.as_raw_window();
        unsafe {
            let mut rect = MaybeUninit::uninit();
            syscall!(BOOL, GetWindowRect(handle, rect.as_mut_ptr())).unwrap();
            let rect = rect.assume_init();
            let mut point = POINT {
                x: rect.left,
                y: rect.top,
            };
            SetLastError(0);
            match syscall!(
                BOOL,
                MapWindowPoints(HWND_DESKTOP, GetParent(handle), &mut point, 2,)
            ) {
                Ok(_) => {}
                Err(e) if e.raw_os_error() == Some(0) => {}
                Err(e) => panic!("{e:?}"),
            }
            (point.x, point.y)
        }
    }

    fn set_locd(&mut self, p: (i32, i32)) {
        let handle = self.as_raw_window();
        if p != self.locd() {
            syscall!(
                BOOL,
                SetWindowPos(
                    handle,
                    null_mut(),
                    p.0,
                    p.1,
                    0,
                    0,
                    SWP_NOSIZE | SWP_NOZORDER,
                )
            )
            .unwrap();
        }
    }

    pub fn loc(&self) -> Point {
        self.point_d2l(self.locd())
    }

    pub fn set_loc(&mut self, p: Point) {
        self.set_locd(self.point_l2d(p))
    }

    pub fn is_visible(&self) -> bool {
        unsafe { IsWindowVisible(self.as_raw_window()) != 0 }
    }

    pub fn set_visible(&mut self, v: bool) {
        unsafe {
            ShowWindow(self.as_raw_window(), if v { SW_SHOW } else { SW_HIDE });
        }
    }

    pub fn is_enabled(&self) -> bool {
        unsafe { IsWindowEnabled(self.as_raw_window()) != 0 }
    }

    pub fn set_enabled(&mut self, v: bool) {
        unsafe {
            EnableWindow(self.as_raw_window(), if v { 1 } else { 0 });
        }
    }

    pub fn text(&self) -> String {
        self.text_u16().to_string_lossy()
    }

    pub fn set_text(&mut self, s: impl AsRef<str>) {
        let handle = self.as_raw_window();
        let s = U16CString::from_str_truncate(s);
        syscall!(BOOL, unsafe { SetWindowTextW(handle, s.as_ptr()) }).unwrap();
    }

    pub fn text_u16(&self) -> U16CString {
        let handle = self.as_raw_window();
        let len = unsafe { GetWindowTextLengthW(handle) };
        if len == 0 {
            return U16CString::new();
        };
        let mut res: Vec<u16> = Vec::with_capacity(len as usize + 1);
        syscall!(BOOL, unsafe {
            GetWindowTextW(handle, res.as_mut_ptr(), res.capacity() as _)
        })
        .unwrap();
        unsafe { res.set_len(len as usize + 1) };
        unsafe { U16CString::from_vec_unchecked(res) }
    }

    pub fn style(&self) -> u32 {
        syscall!(
            BOOL,
            GetWindowLongPtrW(self.as_raw_window(), GWL_STYLE) as u32
        )
        .unwrap()
    }

    pub fn set_style(&mut self, style: u32) {
        unsafe { SetLastError(0) };
        let res = syscall!(
            BOOL,
            SetWindowLongPtrW(self.as_raw_window(), GWL_STYLE, style as _) as i32
        );
        match res {
            Ok(_) => {}
            Err(e) if e.raw_os_error() == Some(0) => {}
            Err(e) => panic!("{e:?}"),
        }
    }

    pub fn set_icon(&mut self, icon: HICON) {
        unsafe {
            SendMessageW(self.as_raw_window(), WM_SETICON, ICON_BIG as _, icon as _);
        }
    }
}

impl AsRawWindow for Widget {
    fn as_raw_window(&self) -> RawWindow {
        self.0.as_raw_window()
    }
}

pub const WINDOW_CLASS_NAME: *const u16 = w!("XamlWindow");

fn register() {
    unsafe {
        set_preferred_app_mode(PreferredAppMode::AllowDark);
    }
    let cls = WNDCLASSEXW {
        cbSize: std::mem::size_of::<WNDCLASSEXW>() as _,
        style: 0,
        lpfnWndProc: Some(window_proc),
        cbClsExtra: 0,
        cbWndExtra: 0,
        hInstance: unsafe { GetModuleHandleW(null()) },
        hIcon: null_mut(),
        hCursor: unsafe { LoadCursorW(null_mut(), IDC_ARROW) },
        hbrBackground: null_mut(),
        lpszMenuName: null(),
        lpszClassName: WINDOW_CLASS_NAME,
        hIconSm: null_mut(),
    };
    syscall!(BOOL, unsafe { RegisterClassExW(&cls) }).unwrap();
}

static REGISTER: OnceLock<()> = OnceLock::new();

fn register_once() {
    REGISTER.get_or_init(register);
}

#[derive(Debug)]
pub struct Window {
    handle: Widget,
}

impl Window {
    pub fn new(parent: Option<impl AsWindow>) -> Self {
        register_once();
        let handle = if let Some(parent) = parent {
            Widget::new(
                WINDOW_CLASS_NAME,
                WS_OVERLAPPEDWINDOW | WS_CHILDWINDOW,
                0,
                parent.as_window().as_raw_window(),
            )
        } else {
            Widget::new(WINDOW_CLASS_NAME, WS_OVERLAPPEDWINDOW, 0, null_mut())
        };
        let this = Self { handle };
        unsafe { window_use_dark_mode(this.as_raw_window()) };
        this
    }

    pub fn as_raw_window(&self) -> HWND {
        self.handle.as_raw_window()
    }

    pub fn is_visible(&self) -> bool {
        self.handle.is_visible()
    }

    pub fn set_visible(&mut self, v: bool) {
        self.handle.set_visible(v);
    }

    pub fn loc(&self) -> Point {
        self.handle.loc()
    }

    pub fn set_loc(&mut self, p: Point) {
        self.handle.set_loc(p)
    }

    pub fn size(&self) -> Size {
        self.handle.size()
    }

    pub fn set_size(&mut self, v: Size) {
        self.handle.set_size(v)
    }

    pub fn client_size(&self) -> Size {
        let handle = self.as_raw_window();
        let mut rect = MaybeUninit::uninit();
        syscall!(BOOL, unsafe { GetClientRect(handle, rect.as_mut_ptr()) }).unwrap();
        let rect = unsafe { rect.assume_init() };
        self.handle
            .size_d2l((rect.right - rect.left, rect.bottom - rect.top))
    }

    pub fn text(&self) -> String {
        self.handle.text()
    }

    pub fn set_text(&mut self, s: impl AsRef<str>) {
        self.handle.set_text(s)
    }

    pub fn set_icon_by_id(&mut self, id: u16) {
        let icon = unsafe {
            LoadImageW(
                GetModuleHandleW(null()),
                id as _,
                IMAGE_ICON,
                0,
                0,
                LR_DEFAULTCOLOR | LR_DEFAULTSIZE | LR_SHARED,
            )
        };
        if icon.is_null() {
            panic!("{:?}", std::io::Error::last_os_error());
        }
        self.handle.set_icon(icon);
    }

    pub fn style(&self) -> u32 {
        self.handle.style()
    }

    pub fn set_style(&mut self, v: u32) {
        self.handle.set_style(v);
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
    fn as_raw_window(&self) -> RawWindow {
        self.handle.as_raw_window()
    }
}

impl Drop for Window {
    fn drop(&mut self) {
        unsafe {
            DestroyWindow(self.handle.as_raw_window());
        }
    }
}
