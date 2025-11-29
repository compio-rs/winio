#[cfg(feature = "once_cell_try")]
use std::cell::OnceCell;
use std::{cell::RefCell, mem::MaybeUninit, rc::Rc, sync::Arc};

use compio::driver::syscall;
use compio_log::error;
use inherit_methods_macro::inherit_methods;
#[cfg(not(feature = "once_cell_try"))]
use once_cell::unsync::OnceCell;
use send_wrapper::SendWrapper;
use windows::{
    Foundation::TypedEventHandler,
    UI::ViewManagement::UISettings,
    Win32::Foundation::E_NOINTERFACE,
    core::{Interface, Ref},
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

use crate::{CustomDesktopAcrylicBackdrop, Error, GlobalRuntime, Result, Widget, ui::Convertible};

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
    pub fn new() -> Result<Self> {
        let handle = MUX::Window::new()?;
        ROOT_WINDOWS.with_borrow_mut(|map| map.push(handle.clone()));

        let hwnd = unsafe { handle.cast::<IWindowNative>()?.WindowHandle()? };
        let app_window = AppWindow::GetFromWindowId(WindowId { Value: hwnd.0 as _ })?;
        let titlebar = app_window.TitleBar()?;
        match titlebar.SetPreferredTheme(TitleBarTheme::UseDefaultAppMode) {
            Ok(()) => {}
            // Available since 1.7
            Err(e) if e.code() == E_NOINTERFACE => unsafe {
                window_use_dark_mode(hwnd.0)?;
                // Set to DWMSBT_AUTO.
                set_backdrop(hwnd.0, Backdrop::None)?;
            },
            Err(e) => return Err(e),
        }

        let canvas = MUXC::Canvas::new()?;
        canvas.SetVerticalAlignment(MUX::VerticalAlignment::Stretch)?;
        canvas.SetHorizontalAlignment(MUX::HorizontalAlignment::Stretch)?;

        handle.SetContent(&canvas)?;

        let on_close = SendWrapper::new(Rc::new(Callback::new()));
        {
            let on_close = on_close.clone();
            app_window.Closing(&TypedEventHandler::new(
                move |_, args: Ref<AppWindowClosingEventArgs>| {
                    if let Some(args) = args.as_ref() {
                        let handled = !on_close.signal::<GlobalRuntime>(());
                        args.SetCancel(handled)?;
                    }
                    Ok(())
                },
            ))?;
        }
        let on_size = SendWrapper::new(Rc::new(Callback::new()));
        let on_move = SendWrapper::new(Rc::new(Callback::new()));
        {
            let on_size = on_size.clone();
            let on_move = on_move.clone();
            app_window.Changed(&TypedEventHandler::new(
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
            ))?;
        }
        {
            let on_size = on_size.clone();
            canvas.Loaded(&RoutedEventHandler::new(move |_, _| {
                on_size.signal::<GlobalRuntime>(());
                Ok(())
            }))?;
        }
        let theme_watcher = ColorThemeWatcher::new()?;

        Ok(Self {
            on_size,
            on_move,
            on_close,
            theme_watcher,
            handle,
            app_window,
            canvas,
        })
    }

    pub fn is_visible(&self) -> Result<bool> {
        self.app_window.IsVisible()
    }

    pub fn set_visible(&self, v: bool) -> Result<()> {
        if v {
            self.app_window.Show()?;
        } else {
            self.app_window.Hide()?;
        }
        Ok(())
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

    pub fn loc(&self) -> Result<Point> {
        Ok(Point::from_native(self.app_window.Position()?) / self.scale())
    }

    pub fn set_loc(&mut self, p: Point) -> Result<()> {
        self.app_window.Move((p * self.scale()).to_native())?;
        Ok(())
    }

    pub fn size(&self) -> Result<Size> {
        Ok(Size::from_native(self.app_window.Size()?) / self.scale())
    }

    pub fn set_size(&mut self, s: Size) -> Result<()> {
        self.app_window.Resize((s * self.scale()).to_native())?;
        Ok(())
    }

    pub fn client_size(&self) -> Result<Size> {
        let size = match self.app_window.ClientSize() {
            Ok(s) => Size::from_native(s),
            // Available since 1.1
            Err(e) if e.code() == E_NOINTERFACE => {
                let mut rect = MaybeUninit::uninit();
                syscall!(
                    BOOL,
                    GetClientRect(self.app_window.Id()?.Value as _, rect.as_mut_ptr())
                )?;
                let rect = unsafe { rect.assume_init() };
                Size::new((rect.right - rect.left) as _, (rect.bottom - rect.top) as _)
            }
            Err(e) => return Err(e),
        };
        Ok(size / self.scale())
    }

    pub fn text(&self) -> Result<String> {
        Ok(self.handle.Title()?.to_string_lossy())
    }

    pub fn set_text(&mut self, text: impl AsRef<str>) -> Result<()> {
        self.handle.SetTitle(&text.as_ref().into())?;
        Ok(())
    }

    pub fn set_icon_by_id(&mut self, id: u16) -> Result<()> {
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
            return Err(Error::from_thread());
        }
        self.app_window
            .SetIconWithIconId(IconId { Value: icon as _ })?;
        Ok(())
    }

    pub fn backdrop(&self) -> Result<Backdrop> {
        match self.handle.SystemBackdrop() {
            Ok(brush) => {
                if let Ok(brush) = brush.cast::<MicaBackdrop>() {
                    match brush.Kind() {
                        Ok(MicaKind::Base) => Ok(Backdrop::Mica),
                        Ok(MicaKind::BaseAlt) => Ok(Backdrop::MicaAlt),
                        _ => Ok(Backdrop::None),
                    }
                } else {
                    Ok(Backdrop::Acrylic)
                }
            }
            Err(e) if e.code().0 == 0 => Ok(Backdrop::None),
            Err(e) => Err(e),
        }
    }

    pub fn set_backdrop(&mut self, backdrop: Backdrop) -> Result<()> {
        match backdrop {
            Backdrop::Acrylic => {
                let brush = acrylic_backdrop()?;
                self.handle.SetSystemBackdrop(&brush)?;
            }
            Backdrop::Mica => {
                let brush = mica_backdrop()?;
                self.handle.SetSystemBackdrop(&brush)?;
            }
            Backdrop::MicaAlt => {
                let brush = mica_alt_backdrop()?;
                self.handle.SetSystemBackdrop(&brush)?;
            }
            _ => {
                self.handle.SetSystemBackdrop(None)?;
            }
        }
        unsafe {
            let hwnd = self.app_window.Id()?.Value as _;
            set_backdrop(hwnd, backdrop)?;
        }
        Ok(())
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
            if let Ok(c) = w.Content()
                && let Ok(r) = c.XamlRoot()
                && r == e_root
            {
                return Some(w.clone());
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
    pub fn new() -> Result<Self> {
        let settings = UISettings::new()?;
        let notify = Arc::new(SyncCallback::new());
        let token = {
            let notify = notify.clone();
            settings.ColorValuesChanged(&TypedEventHandler::new(move |_, _| {
                notify.signal(());
                Ok(())
            }))?
        };
        Ok(Self {
            settings,
            notify,
            token,
        })
    }

    pub async fn wait(&self) {
        self.notify.wait().await
    }
}

impl Drop for ColorThemeWatcher {
    fn drop(&mut self) {
        match self.settings.RemoveColorValuesChanged(self.token) {
            Ok(()) => {}
            Err(_e) => {
                error!("RemoveColorValuesChanged: {_e:?}");
            }
        }
    }
}

fn acrylic_backdrop() -> Result<SystemBackdrop> {
    thread_local! {
        static ACRYLIC_BACKDROP: OnceCell<SystemBackdrop> = const { OnceCell::new() };
    }

    ACRYLIC_BACKDROP.with(|cell| {
        cell.get_or_try_init(CustomDesktopAcrylicBackdrop::compose)
            .cloned()
    })
}

fn mica_backdrop() -> Result<MicaBackdrop> {
    thread_local! {
        static MICA_BACKDROP: OnceCell<MicaBackdrop> = const { OnceCell::new() };
    }

    MICA_BACKDROP.with(|cell| {
        cell.get_or_try_init(|| {
            let brush = MicaBackdrop::new()?;
            brush.SetKind(MicaKind::Base)?;
            Ok(brush)
        })
        .cloned()
    })
}

fn mica_alt_backdrop() -> Result<MicaBackdrop> {
    thread_local! {
        static MICA_ALT_BACKDROP: OnceCell<MicaBackdrop> = const { OnceCell::new() };
    }

    MICA_ALT_BACKDROP.with(|cell| {
        cell.get_or_try_init(|| {
            let brush = MicaBackdrop::new()?;
            brush.SetKind(MicaKind::BaseAlt)?;
            Ok(brush)
        })
        .cloned()
    })
}

#[derive(Debug)]
pub struct View {
    handle: Widget,
    canvas: MUXC::Canvas,
}

#[inherit_methods(from = "self.handle")]
impl View {
    pub fn new(parent: impl AsContainer) -> Result<Self> {
        let canvas = MUXC::Canvas::new()?;
        Ok(Self {
            handle: Widget::new(parent, canvas.cast()?)?,
            canvas,
        })
    }

    pub fn is_visible(&self) -> Result<bool>;

    pub fn set_visible(&mut self, v: bool) -> Result<()>;

    pub fn is_enabled(&self) -> Result<bool>;

    pub fn set_enabled(&mut self, v: bool) -> Result<()>;

    pub fn loc(&self) -> Result<Point>;

    pub fn set_loc(&mut self, p: Point) -> Result<()>;

    pub fn size(&self) -> Result<Size>;

    pub fn set_size(&mut self, v: Size) -> Result<()>;
}

winio_handle::impl_as_widget!(View, handle);

impl AsRawContainer for View {
    fn as_raw_container(&self) -> RawContainer {
        RawContainer::WinUI(self.canvas.clone())
    }
}

winio_handle::impl_as_container!(View);
