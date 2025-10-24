use std::{
    cell::{OnceCell, RefCell},
    mem::MaybeUninit,
    rc::Rc,
    sync::Arc,
};

use inherit_methods_macro::inherit_methods;
use send_wrapper::SendWrapper;
use windows::{
    Foundation::TypedEventHandler,
    UI::ViewManagement::UISettings,
    Win32::Foundation::{E_NOINTERFACE, REGDB_E_CLASSNOTREG},
    core::{Interface, Ref, Result},
};
use windows_sys::Win32::UI::{
    HiDpi::GetDpiForWindow,
    WindowsAndMessaging::{
        GetClientRect, IMAGE_ICON, LR_DEFAULTCOLOR, LR_DEFAULTSIZE, LR_SHARED, LoadImageW,
    },
};
use winio_callback::{Callback, SyncCallback};
use winio_handle::{AsContainer, AsRawContainer, AsRawWindow, RawContainer, RawWindow};
use winio_primitive::{Point, Size};
use winio_ui_windows_common::{
    Backdrop, get_current_module_handle, set_backdrop, window_use_dark_mode,
};
use winui3::{
    IWindowNative,
    Microsoft::UI::{
        Composition::SystemBackdrops::MicaKind,
        IconId, WindowId,
        Windowing::{
            AppWindow, AppWindowChangedEventArgs, AppWindowClosingEventArgs, TitleBarTheme,
        },
        Xaml::{
            self as MUX, Controls as MUXC,
            Media::{MicaBackdrop, SystemBackdrop},
            RoutedEventHandler,
        },
    },
};

use crate::{CustomDesktopAcrylicBackdrop, GlobalRuntime, Widget, ui::Convertible};

#[derive(Debug)]
pub struct Window {
    on_size: SendWrapper<Rc<Callback>>,
    on_move: SendWrapper<Rc<Callback>>,
    on_close: SendWrapper<Rc<Callback>>,
    theme_watcher: ColorThemeWatcher,
    handle: MUX::Window,
    app_window: AppWindow,
    canvas: MUXC::Canvas,
}

impl Window {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        let handle = MUX::Window::new().unwrap();
        ROOT_WINDOWS.with_borrow_mut(|map| map.push(handle.clone()));

        let hwnd = unsafe {
            handle
                .cast::<IWindowNative>()
                .unwrap()
                .WindowHandle()
                .unwrap()
        };
        let app_window = AppWindow::GetFromWindowId(WindowId { Value: hwnd.0 as _ }).unwrap();
        let titlebar = app_window.TitleBar().unwrap();
        match titlebar.SetPreferredTheme(TitleBarTheme::UseDefaultAppMode) {
            Ok(()) => {}
            // Available since 1.7
            Err(e) if e.code() == E_NOINTERFACE => unsafe {
                window_use_dark_mode(hwnd.0);
                // Set to DWMSBT_AUTO.
                set_backdrop(hwnd.0, Backdrop::None);
            },
            Err(e) => panic!("{e:?}"),
        }

        let canvas = MUXC::Canvas::new().unwrap();
        canvas
            .SetVerticalAlignment(MUX::VerticalAlignment::Stretch)
            .unwrap();
        canvas
            .SetHorizontalAlignment(MUX::HorizontalAlignment::Stretch)
            .unwrap();

        handle.SetContent(&canvas).unwrap();

        let on_close = SendWrapper::new(Rc::new(Callback::new()));
        {
            let on_close = on_close.clone();
            app_window
                .Closing(&TypedEventHandler::new(
                    move |_, args: Ref<AppWindowClosingEventArgs>| {
                        if let Some(args) = args.as_ref() {
                            let handled = !on_close.signal::<GlobalRuntime>(());
                            args.SetCancel(handled)?;
                        }
                        Ok(())
                    },
                ))
                .unwrap();
        }
        let on_size = SendWrapper::new(Rc::new(Callback::new()));
        let on_move = SendWrapper::new(Rc::new(Callback::new()));
        {
            let on_size = on_size.clone();
            let on_move = on_move.clone();
            app_window
                .Changed(&TypedEventHandler::new(
                    move |_, args: Ref<AppWindowChangedEventArgs>| {
                        if let Some(args) = args.as_ref() {
                            if args.DidPositionChange()? {
                                on_move.signal::<GlobalRuntime>(());
                            }
                            if args.DidSizeChange()? {
                                on_size.signal::<GlobalRuntime>(());
                            }
                        }
                        Ok(())
                    },
                ))
                .unwrap();
        }
        {
            let on_size = on_size.clone();
            canvas
                .Loaded(&RoutedEventHandler::new(move |_, _| {
                    on_size.signal::<GlobalRuntime>(());
                    Ok(())
                }))
                .unwrap();
        }
        let theme_watcher = ColorThemeWatcher::new();

        Self {
            on_size,
            on_move,
            on_close,
            theme_watcher,
            handle,
            app_window,
            canvas,
        }
    }

    pub fn is_visible(&self) -> bool {
        self.app_window.IsVisible().unwrap()
    }

    pub fn set_visible(&self, v: bool) {
        if v {
            self.app_window.Show().unwrap();
        } else {
            self.app_window.Hide().unwrap();
        }
    }

    fn dpi(&self) -> u32 {
        if let Ok(id) = self.app_window.Id() {
            unsafe { GetDpiForWindow(id.Value as _) }
        } else {
            96
        }
    }

    fn scale(&self) -> f64 {
        self.dpi() as f64 / 96.0
    }

    pub fn loc(&self) -> Point {
        Point::from_native(self.app_window.Position().unwrap()) / self.scale()
    }

    pub fn set_loc(&mut self, p: Point) {
        self.app_window
            .Move((p * self.scale()).to_native())
            .unwrap();
    }

    pub fn size(&self) -> Size {
        Size::from_native(self.app_window.Size().unwrap()) / self.scale()
    }

    pub fn set_size(&mut self, s: Size) {
        self.app_window
            .Resize((s * self.scale()).to_native())
            .unwrap();
    }

    pub fn client_size(&self) -> Size {
        let size = match self.app_window.ClientSize() {
            Ok(s) => Size::from_native(s),
            // Available since 1.1
            Err(e) if e.code() == E_NOINTERFACE => {
                let mut rect = MaybeUninit::uninit();
                if unsafe {
                    GetClientRect(self.app_window.Id().unwrap().Value as _, rect.as_mut_ptr())
                } == 0
                {
                    panic!("{:?}", std::io::Error::last_os_error());
                } else {
                    let rect = unsafe { rect.assume_init() };
                    Size::new((rect.right - rect.left) as _, (rect.bottom - rect.top) as _)
                }
            }
            Err(e) => panic!("{e:?}"),
        };
        size / self.scale()
    }

    pub fn text(&self) -> String {
        self.handle.Title().unwrap().to_string_lossy()
    }

    pub fn set_text(&mut self, text: impl AsRef<str>) {
        self.handle.SetTitle(&text.as_ref().into()).unwrap();
    }

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
        self.app_window
            .SetIconWithIconId(IconId { Value: icon as _ })
            .unwrap();
    }

    pub fn backdrop(&self) -> Backdrop {
        match self.handle.SystemBackdrop() {
            Ok(brush) => {
                if let Ok(brush) = brush.cast::<MicaBackdrop>() {
                    match brush.Kind() {
                        Ok(MicaKind::Base) => Backdrop::Mica,
                        Ok(MicaKind::BaseAlt) => Backdrop::MicaAlt,
                        _ => Backdrop::None,
                    }
                } else {
                    Backdrop::Acrylic
                }
            }
            Err(_) => Backdrop::None,
        }
    }

    pub fn set_backdrop(&mut self, backdrop: Backdrop) {
        match self.set_backdrop_impl(backdrop) {
            Ok(_) => {}
            // Available since 1.3
            Err(e) if matches!(e.code(), E_NOINTERFACE | REGDB_E_CLASSNOTREG) => return,
            Err(e) => panic!("{e:?}"),
        }
        unsafe {
            let hwnd = self.app_window.Id().unwrap().Value as _;
            set_backdrop(hwnd, backdrop);
        }
    }

    fn set_backdrop_impl(&mut self, backdrop: Backdrop) -> Result<bool> {
        match backdrop {
            Backdrop::Acrylic => {
                let brush = acrylic_backdrop()?;
                self.handle.SetSystemBackdrop(&brush)?;
                Ok(true)
            }
            Backdrop::Mica => {
                let brush = mica_backdrop()?;
                self.handle.SetSystemBackdrop(&brush)?;
                Ok(true)
            }
            Backdrop::MicaAlt => {
                let brush = mica_alt_backdrop()?;
                self.handle.SetSystemBackdrop(&brush)?;
                Ok(true)
            }
            _ => {
                self.handle.SetSystemBackdrop(None)?;
                Ok(false)
            }
        }
    }

    pub async fn wait_size(&self) {
        self.on_size.wait().await
    }

    pub async fn wait_move(&self) {
        self.on_move.wait().await
    }

    pub async fn wait_close(&self) {
        self.on_close.wait().await
    }

    pub async fn wait_theme_changed(&self) {
        self.theme_watcher.wait().await
    }
}

impl AsRawWindow for Window {
    fn as_raw_window(&self) -> RawWindow {
        RawWindow::WinUI(self.handle.clone())
    }
}

winio_handle::impl_as_window!(Window);

impl AsRawContainer for Window {
    fn as_raw_container(&self) -> RawContainer {
        RawContainer::WinUI(self.canvas.clone())
    }
}

winio_handle::impl_as_container!(Window);

impl Drop for Window {
    fn drop(&mut self) {
        ROOT_WINDOWS.with_borrow_mut(|map| {
            map.retain(|w| w != &self.handle);
        });
    }
}

thread_local! {
    static ROOT_WINDOWS: RefCell<Vec<MUX::Window>> = const { RefCell::new(vec![]) };
}

pub(crate) fn get_root_window(e: &MUX::FrameworkElement) -> Option<MUX::Window> {
    let e_root = e.XamlRoot().ok()?;
    ROOT_WINDOWS.with_borrow(|windows| {
        for w in windows {
            if let Ok(c) = w.Content() {
                if let Ok(r) = c.XamlRoot() {
                    if r == e_root {
                        return Some(w.clone());
                    }
                }
            }
        }
        None
    })
}

#[derive(Debug)]
struct ColorThemeWatcher {
    settings: UISettings,
    notify: Arc<SyncCallback>,
    token: i64,
}

impl ColorThemeWatcher {
    pub fn new() -> Self {
        let settings = UISettings::new().unwrap();
        let notify = Arc::new(SyncCallback::new());
        let token = {
            let notify = notify.clone();
            settings
                .ColorValuesChanged(&TypedEventHandler::new(move |_, _| {
                    notify.signal(());
                    Ok(())
                }))
                .unwrap()
        };
        Self {
            settings,
            notify,
            token,
        }
    }

    pub async fn wait(&self) {
        self.notify.wait().await
    }
}

impl Drop for ColorThemeWatcher {
    fn drop(&mut self) {
        self.settings.RemoveColorValuesChanged(self.token).unwrap();
    }
}

fn acrylic_backdrop() -> Result<SystemBackdrop> {
    thread_local! {
        static ACRYLIC_BACKDROP: OnceCell<Result<SystemBackdrop>> = const { OnceCell::new() };
    }

    ACRYLIC_BACKDROP.with(|cell| {
        cell.get_or_init(CustomDesktopAcrylicBackdrop::compose)
            .clone()
    })
}

fn mica_backdrop() -> Result<MicaBackdrop> {
    thread_local! {
        static MICA_BACKDROP: OnceCell<Result<MicaBackdrop>> = const { OnceCell::new() };
    }

    MICA_BACKDROP.with(|cell| {
        cell.get_or_init(|| {
            let brush = MicaBackdrop::new()?;
            brush.SetKind(MicaKind::Base)?;
            Ok(brush)
        })
        .clone()
    })
}

fn mica_alt_backdrop() -> Result<MicaBackdrop> {
    thread_local! {
        static MICA_ALT_BACKDROP: OnceCell<Result<MicaBackdrop>> = const { OnceCell::new() };
    }

    MICA_ALT_BACKDROP.with(|cell| {
        cell.get_or_init(|| {
            let brush = MicaBackdrop::new()?;
            brush.SetKind(MicaKind::BaseAlt)?;
            Ok(brush)
        })
        .clone()
    })
}

#[derive(Debug)]
pub struct View {
    handle: Widget,
    canvas: MUXC::Canvas,
}

#[inherit_methods(from = "self.handle")]
impl View {
    pub fn new(parent: impl AsContainer) -> Self {
        let canvas = MUXC::Canvas::new().unwrap();
        Self {
            handle: Widget::new(parent, canvas.cast().unwrap()),
            canvas,
        }
    }

    pub fn is_visible(&self) -> bool;

    pub fn set_visible(&mut self, v: bool);

    pub fn is_enabled(&self) -> bool;

    pub fn set_enabled(&mut self, v: bool);

    pub fn loc(&self) -> Point;

    pub fn set_loc(&mut self, p: Point);

    pub fn size(&self) -> Size;

    pub fn set_size(&mut self, v: Size);
}

winio_handle::impl_as_widget!(View, handle);

impl AsRawContainer for View {
    fn as_raw_container(&self) -> RawContainer {
        RawContainer::WinUI(self.canvas.clone())
    }
}

winio_handle::impl_as_container!(View);
