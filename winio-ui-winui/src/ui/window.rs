use std::rc::Rc;

use send_wrapper::SendWrapper;
use windows::{Foundation::TypedEventHandler, core::Ref};
use windows_sys::Win32::UI::{
    HiDpi::GetDpiForWindow,
    WindowsAndMessaging::{IMAGE_ICON, LR_DEFAULTCOLOR, LR_DEFAULTSIZE, LR_SHARED, LoadImageW},
};
use winio_callback::Callback;
use winio_handle::{AsRawWindow, AsWindow, BorrowedWindow, RawWindow};
use winio_primitive::{Point, Size};
use winio_ui_windows_common::get_current_module_handle;
use winui3::Microsoft::UI::{
    IconId,
    Windowing::{AppWindow, AppWindowChangedEventArgs, AppWindowClosingEventArgs, TitleBarTheme},
    Xaml::{self as MUX, Controls as MUXC, RoutedEventHandler},
};

use crate::{GlobalRuntime, ui::Convertible};

#[derive(Debug)]
pub struct Window {
    on_size: SendWrapper<Rc<Callback>>,
    on_move: SendWrapper<Rc<Callback>>,
    on_close: SendWrapper<Rc<Callback>>,
    handle: MUX::Window,
    app_window: AppWindow,
    #[allow(dead_code)]
    canvas: MUXC::Canvas,
}

impl Window {
    pub fn new(_parent: Option<impl AsWindow>) -> Self {
        let handle = MUX::Window::new().unwrap();

        let app_window = handle.AppWindow().unwrap();
        let titlebar = app_window.TitleBar().unwrap();
        titlebar
            .SetPreferredTheme(TitleBarTheme::UseDefaultAppMode)
            .unwrap();

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
                            args.SetCancel(handled).unwrap();
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

        Self {
            on_size,
            on_move,
            on_close,
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
        Size::from_native(self.app_window.ClientSize().unwrap()) / self.scale()
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

    pub async fn wait_size(&self) {
        self.on_size.wait().await
    }

    pub async fn wait_move(&self) {
        self.on_move.wait().await
    }

    pub async fn wait_close(&self) {
        self.on_close.wait().await
    }
}

impl AsRawWindow for Window {
    fn as_raw_window(&self) -> RawWindow {
        RawWindow::WinUI(self.handle.clone())
    }
}

impl AsWindow for Window {
    fn as_window(&self) -> BorrowedWindow<'_> {
        unsafe { BorrowedWindow::borrow_raw(self.as_raw_window()) }
    }
}
