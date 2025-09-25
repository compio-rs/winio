use std::{
    mem::MaybeUninit,
    ptr::{null, null_mut},
    sync::Once,
};

use compio::driver::syscall;
use inherit_methods_macro::inherit_methods;
use widestring::{U16CStr, U16CString, U16Str, u16cstr};
use windows_sys::Win32::{
    Foundation::{ERROR_INVALID_HANDLE, HWND, LPARAM, LRESULT, SetLastError, WPARAM},
    Graphics::Gdi::{GetStockObject, MapWindowPoints, WHITE_BRUSH},
    UI::{
        Input::KeyboardAndMouse::{EnableWindow, IsWindowEnabled},
        WindowsAndMessaging::{
            CW_USEDEFAULT, CreateWindowExW, DestroyWindow, GWL_EXSTYLE, GWL_STYLE, GetClientRect,
            GetParent, GetWindowLongPtrW, GetWindowRect, GetWindowTextLengthW, GetWindowTextW,
            HICON, HWND_DESKTOP, ICON_BIG, IDC_ARROW, IMAGE_ICON, LR_DEFAULTCOLOR, LR_DEFAULTSIZE,
            LR_SHARED, LoadCursorW, LoadImageW, RegisterClassExW, SW_HIDE, SW_SHOW, SWP_NOMOVE,
            SWP_NOSIZE, SWP_NOZORDER, SendMessageW, SetWindowLongPtrW, SetWindowPos,
            SetWindowTextW, ShowWindow, WM_CLOSE, WM_MOVE, WM_SETICON, WM_SIZE, WNDCLASSEXW,
            WS_CHILDWINDOW, WS_CLIPCHILDREN, WS_EX_CONTROLPARENT, WS_OVERLAPPEDWINDOW, WS_VISIBLE,
        },
    },
};
use winio_handle::{
    AsContainer, AsRawContainer, AsRawWidget, AsRawWindow, RawContainer, RawWidget, RawWindow,
};
use winio_primitive::{Point, Size};
use winio_ui_windows_common::{
    PreferredAppMode, control_use_dark_mode, get_current_module_handle, set_preferred_app_mode,
    window_use_dark_mode,
};

use crate::{
    font::measure_string,
    runtime::{WindowMessage, wait, window_proc},
    tooltip::{get_tooltip, remove_tooltip, set_tooltip},
    ui::{
        dpi::{DpiAware, get_dpi_for_window},
        get_u16c, with_u16c,
    },
};

#[derive(Debug)]
pub(crate) struct OwnedWindow(HWND);

impl OwnedWindow {
    pub unsafe fn from_raw_window(h: HWND) -> Self {
        Self(h)
    }
}

impl Drop for OwnedWindow {
    fn drop(&mut self) {
        unsafe { DestroyWindow(self.0) };
    }
}

impl AsRawWindow for OwnedWindow {
    fn as_raw_window(&self) -> RawWindow {
        RawWindow::Win32(self.0)
    }
}

impl AsRawWidget for OwnedWindow {
    fn as_raw_widget(&self) -> RawWidget {
        RawWidget::Win32(self.0)
    }
}

impl AsRawContainer for OwnedWindow {
    fn as_raw_container(&self) -> RawContainer {
        RawContainer::Win32(self.0)
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
                get_current_module_handle(),
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

    pub async fn wait(&self, msg: u32) -> WindowMessage {
        unsafe { wait(self.as_raw_window().as_win32(), msg) }.await
    }

    pub async fn wait_parent(&self, msg: u32) -> WindowMessage {
        unsafe { wait(GetParent(self.as_raw_window().as_win32()), msg) }.await
    }

    pub fn measure(&self, s: &U16Str) -> Size {
        measure_string(self.as_raw_window().as_win32(), s)
    }

    pub fn measure_text(&self) -> Size {
        self.measure(self.text_u16().as_ustr())
    }

    pub fn dpi(&self) -> u32 {
        unsafe { get_dpi_for_window(self.as_raw_window().as_win32()) }
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
        let handle = self.as_raw_window().as_win32();
        let mut rect = MaybeUninit::uninit();
        syscall!(BOOL, unsafe { GetWindowRect(handle, rect.as_mut_ptr()) }).unwrap();
        let rect = unsafe { rect.assume_init() };
        (rect.right - rect.left, rect.bottom - rect.top)
    }

    fn set_sized(&mut self, v: (i32, i32)) {
        let handle = self.as_raw_window().as_win32();
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
        let handle = self.as_raw_window().as_win32();
        unsafe {
            let mut rect = MaybeUninit::uninit();
            syscall!(BOOL, GetWindowRect(handle, rect.as_mut_ptr())).unwrap();
            let mut rect = rect.assume_init();
            SetLastError(0);
            match syscall!(
                BOOL,
                MapWindowPoints(HWND_DESKTOP, GetParent(handle), &mut rect as *mut _ as _, 2)
            ) {
                Ok(_) => {}
                Err(e) if e.raw_os_error() == Some(0) => {}
                Err(e) => panic!("{e:?}"),
            }
            (rect.left, rect.right)
        }
    }

    fn set_locd(&mut self, p: (i32, i32)) {
        let handle = self.as_raw_window().as_win32();
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
        (self.style() & WS_VISIBLE) != 0
    }

    pub fn set_visible(&mut self, v: bool) {
        unsafe {
            ShowWindow(
                self.as_raw_window().as_win32(),
                if v { SW_SHOW } else { SW_HIDE },
            );
        }
    }

    pub fn is_enabled(&self) -> bool {
        unsafe { IsWindowEnabled(self.as_raw_window().as_win32()) != 0 }
    }

    pub fn set_enabled(&mut self, v: bool) {
        unsafe {
            EnableWindow(self.as_raw_window().as_win32(), if v { 1 } else { 0 });
        }
    }

    pub fn tooltip(&self) -> String {
        get_tooltip(self.as_raw_window().as_win32()).unwrap_or_default()
    }

    pub fn set_tooltip(&mut self, s: impl AsRef<str>) {
        set_tooltip(self.as_raw_window().as_win32(), s);
    }

    pub fn text(&self) -> String {
        self.text_u16().to_string_lossy()
    }

    pub fn set_text(&mut self, s: impl AsRef<str>) {
        let handle = self.as_raw_window().as_win32();
        with_u16c(s.as_ref(), |s| {
            syscall!(BOOL, unsafe { SetWindowTextW(handle, s.as_ptr()) }).unwrap();
        });
    }

    pub fn text_u16(&self) -> U16CString {
        let handle = self.as_raw_window().as_win32();
        let len = unsafe { GetWindowTextLengthW(handle) };
        unsafe {
            get_u16c(len as usize, |buf| {
                syscall!(
                    BOOL,
                    GetWindowTextW(handle, buf.as_mut_ptr().cast(), buf.len() as _)
                )
                .unwrap() as _
            })
        }
    }

    pub fn style(&self) -> u32 {
        syscall!(
            BOOL,
            GetWindowLongPtrW(self.as_raw_window().as_win32(), GWL_STYLE) as u32
        )
        .unwrap()
    }

    pub fn set_style(&mut self, style: u32) {
        unsafe { SetLastError(0) };
        let res = syscall!(
            BOOL,
            SetWindowLongPtrW(self.as_raw_window().as_win32(), GWL_STYLE, style as _) as i32
        );
        match res {
            Ok(_) => {}
            Err(e) if e.raw_os_error() == Some(0) => {}
            Err(e) => panic!("{e:?}"),
        }
    }

    pub fn ex_style(&self) -> u32 {
        syscall!(
            BOOL,
            GetWindowLongPtrW(self.as_raw_window().as_win32(), GWL_EXSTYLE) as u32
        )
        .unwrap()
    }

    pub fn set_ex_style(&mut self, style: u32) {
        unsafe { SetLastError(0) };
        let res = syscall!(
            BOOL,
            SetWindowLongPtrW(self.as_raw_window().as_win32(), GWL_EXSTYLE, style as _) as i32
        );
        match res {
            Ok(_) => {}
            Err(e)
                if e.raw_os_error() == Some(0)
                    || e.raw_os_error() == Some(ERROR_INVALID_HANDLE as _) => {}
            Err(e) => panic!("{e:?}"),
        }
    }

    pub fn send_message(&self, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        unsafe { SendMessageW(self.as_raw_window().as_win32(), msg, wparam as _, lparam) }
    }

    pub fn set_icon(&mut self, icon: HICON) {
        self.send_message(WM_SETICON, ICON_BIG as _, icon as _);
    }
}

impl Drop for Widget {
    fn drop(&mut self) {
        remove_tooltip(self.as_raw_window().as_win32());
    }
}

impl AsRawWindow for Widget {
    fn as_raw_window(&self) -> RawWindow {
        self.0.as_raw_window()
    }
}

winio_handle::impl_as_window!(Widget);

impl AsRawWidget for Widget {
    fn as_raw_widget(&self) -> RawWidget {
        self.0.as_raw_widget()
    }
}

winio_handle::impl_as_widget!(Widget);

impl AsRawContainer for Widget {
    fn as_raw_container(&self) -> RawContainer {
        self.0.as_raw_container()
    }
}

winio_handle::impl_as_container!(Widget);

const WINDOW_CLASS_NAME: &U16CStr =
    u16cstr!(concat!("WinioWindowVersion", env!("CARGO_PKG_VERSION")));

fn register() {
    set_preferred_app_mode(PreferredAppMode::AllowDark);
    let cls = WNDCLASSEXW {
        cbSize: std::mem::size_of::<WNDCLASSEXW>() as _,
        style: 0,
        lpfnWndProc: Some(window_proc),
        cbClsExtra: 0,
        cbWndExtra: 0,
        hInstance: get_current_module_handle(),
        hIcon: null_mut(),
        hCursor: unsafe { LoadCursorW(null_mut(), IDC_ARROW) },
        hbrBackground: unsafe { GetStockObject(WHITE_BRUSH) },
        lpszMenuName: null(),
        lpszClassName: WINDOW_CLASS_NAME.as_ptr(),
        hIconSm: null_mut(),
    };
    syscall!(BOOL, RegisterClassExW(&cls)).unwrap();
}

static REGISTER: Once = Once::new();

fn register_once() {
    REGISTER.call_once(register);
}

pub(crate) fn window_class_name() -> *const u16 {
    register_once();
    WINDOW_CLASS_NAME.as_ptr()
}

#[derive(Debug)]
pub struct Window {
    handle: Widget,
}

#[inherit_methods(from = "self.handle")]
impl Window {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        let handle = Widget::new(
            window_class_name(),
            WS_OVERLAPPEDWINDOW,
            WS_EX_CONTROLPARENT,
            null_mut(),
        );
        let this = Self { handle };
        unsafe { window_use_dark_mode(this.as_raw_window().as_win32()) };
        this
    }

    pub fn is_visible(&self) -> bool;

    pub fn set_visible(&mut self, v: bool);

    pub fn loc(&self) -> Point;

    pub fn set_loc(&mut self, p: Point);

    pub fn size(&self) -> Size;

    pub fn set_size(&mut self, v: Size);

    pub fn client_size(&self) -> Size {
        let handle = self.as_raw_window().as_win32();
        let mut rect = MaybeUninit::uninit();
        syscall!(BOOL, unsafe { GetClientRect(handle, rect.as_mut_ptr()) }).unwrap();
        let rect = unsafe { rect.assume_init() };
        self.handle
            .size_d2l((rect.right - rect.left, rect.bottom - rect.top))
    }

    pub fn text(&self) -> String;

    pub fn set_text(&mut self, s: impl AsRef<str>);

    pub fn style(&self) -> u32;

    pub fn set_style(&mut self, v: u32);

    pub fn ex_style(&self) -> u32;

    pub fn set_ex_style(&mut self, v: u32);

    pub fn set_icon_by_id(&mut self, id: u16) {
        let icon = unsafe {
            LoadImageW(
                get_current_module_handle(),
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

winio_handle::impl_as_window!(Window, handle);
winio_handle::impl_as_container!(Window, handle);

#[derive(Debug)]
pub struct View {
    handle: Widget,
}

#[inherit_methods(from = "self.handle")]
impl View {
    pub fn new(parent: impl AsContainer) -> Self {
        Self::new_impl(parent.as_container().as_win32(), WS_VISIBLE)
    }

    pub(crate) fn new_hidden(parent: HWND) -> Self {
        Self::new_impl(parent, 0)
    }

    fn new_impl(parent: HWND, style: u32) -> Self {
        let handle = Widget::new(
            window_class_name(),
            WS_CHILDWINDOW | WS_CLIPCHILDREN | style,
            WS_EX_CONTROLPARENT,
            parent,
        );
        let this = Self { handle };
        unsafe { window_use_dark_mode(this.handle.as_raw_window().as_win32()) };
        this
    }

    pub fn is_visible(&self) -> bool;

    pub fn set_visible(&mut self, v: bool);

    pub fn loc(&self) -> Point;

    pub fn set_loc(&mut self, p: Point);

    pub fn size(&self) -> Size;

    pub fn set_size(&mut self, v: Size);
}

winio_handle::impl_as_widget!(View, handle);
winio_handle::impl_as_container!(View, handle);
