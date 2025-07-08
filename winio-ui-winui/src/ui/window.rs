use std::rc::Rc;

use send_wrapper::SendWrapper;
use windows::{Foundation::TypedEventHandler, core::Ref};
use winio_callback::Callback;
use winio_handle::{AsRawWindow, AsWindow, BorrowedWindow, RawWindow};
use winio_primitive::{Point, Size};
use winui3::Microsoft::UI::{
    IconId,
    Windowing::{AppWindow, AppWindowChangedEventArgs, TitleBarTheme},
    Xaml::{self as WUX, Controls as WUXC},
};

use crate::{
    GlobalRuntime, from_graphics_pointi32, from_graphics_sizei32, to_graphics_pointi32,
    to_graphics_sizei32,
};

#[derive(Debug)]
pub struct Window {
    on_changed: SendWrapper<Rc<Callback<Option<AppWindowChangedEventArgs>>>>,
    on_close: SendWrapper<Rc<Callback>>,
    handle: WUX::Window,
    app_window: AppWindow,
    #[allow(dead_code)]
    canvas: WUXC::Canvas,
}

impl Window {
    pub fn new(_parent: Option<impl AsWindow>) -> Self {
        let handle = WUX::Window::new().unwrap();

        let app_window = handle.AppWindow().unwrap();
        let titlebar = app_window.TitleBar().unwrap();
        titlebar
            .SetPreferredTheme(TitleBarTheme::UseDefaultAppMode)
            .unwrap();

        let canvas = WUXC::Canvas::new().unwrap();
        canvas
            .SetVerticalAlignment(WUX::VerticalAlignment::Stretch)
            .unwrap();
        canvas
            .SetHorizontalAlignment(WUX::HorizontalAlignment::Stretch)
            .unwrap();

        handle.SetContent(&canvas).unwrap();

        let on_close = SendWrapper::new(Rc::new(Callback::new()));
        {
            let on_close = on_close.clone();
            handle
                .Closed(&TypedEventHandler::new(
                    move |_, args: Ref<WUX::WindowEventArgs>| {
                        let handled = on_close.signal::<GlobalRuntime>(());
                        args.unwrap().SetHandled(handled).unwrap();
                        Ok(())
                    },
                ))
                .unwrap();
        }
        let on_changed = SendWrapper::new(Rc::new(Callback::new()));
        {
            let on_changed = on_changed.clone();
            app_window
                .Changed(&TypedEventHandler::new(
                    move |_, args: Ref<AppWindowChangedEventArgs>| {
                        on_changed.signal::<GlobalRuntime>(args.clone());
                        Ok(())
                    },
                ))
                .unwrap();
        }

        Self {
            on_changed,
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
            self.handle.Activate().unwrap();
            self.app_window.Show().unwrap();
        } else {
            self.app_window.Hide().unwrap();
        }
    }

    pub fn loc(&self) -> Point {
        from_graphics_pointi32(self.app_window.Position().unwrap())
    }

    pub fn set_loc(&mut self, p: Point) {
        self.app_window.Move(to_graphics_pointi32(p)).unwrap();
    }

    pub fn size(&self) -> Size {
        from_graphics_sizei32(self.app_window.Size().unwrap())
    }

    pub fn set_size(&mut self, s: Size) {
        self.app_window.Resize(to_graphics_sizei32(s)).unwrap();
    }

    pub fn client_size(&self) -> Size {
        from_graphics_sizei32(self.app_window.ClientSize().unwrap())
    }

    pub fn text(&self) -> String {
        self.handle.Title().unwrap().to_string_lossy()
    }

    pub fn set_text(&mut self, text: &str) {
        self.handle.SetTitle(&text.into()).unwrap();
    }

    pub fn set_icon_by_id(&mut self, id: u16) {
        self.app_window
            .SetIconWithIconId(IconId { Value: id as _ })
            .unwrap();
    }

    pub async fn wait_size(&self) {
        loop {
            let args = self.on_changed.wait().await;
            if let Some(args) = args {
                if args.DidSizeChange().unwrap() {
                    break;
                }
            }
        }
    }

    pub async fn wait_move(&self) {
        loop {
            let args = self.on_changed.wait().await;
            if let Some(args) = args {
                if args.DidPositionChange().unwrap() {
                    break;
                }
            }
        }
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
