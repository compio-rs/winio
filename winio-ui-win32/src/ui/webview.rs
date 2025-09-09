use std::future::Future;

use flume::Receiver;
use webview2::{Controller, Environment, WebView as WebView2};
use windows_sys::Win32::{
    Foundation::RECT,
    UI::{HiDpi::GetDpiForWindow, WindowsAndMessaging::IsWindow},
};
use winio_handle::{AsRawWidget, AsWindow, RawWidget};
use winio_primitive::{Point, Rect, Size};
use winio_ui_windows_common::{WebViewImpl, WebViewLazy};

#[derive(Debug)]
pub struct WebViewInner {
    host: Controller,
    view: WebView2,
    nav_rx: Receiver<()>,
}

impl WebViewInner {
    fn dpi(&self) -> f64 {
        let hwnd = self.host.get_parent_window().unwrap();
        unsafe { GetDpiForWindow(hwnd.cast()) as f64 / 96.0 }
    }

    fn rect(&self) -> Rect {
        let rect = self.host.get_bounds().unwrap();
        Rect::new(
            Point::new(rect.left as _, rect.top as _),
            Size::new((rect.right - rect.left) as _, (rect.bottom - rect.top) as _),
        ) / self.dpi()
    }

    #[allow(clippy::missing_transmute_annotations)]
    fn set_rect(&mut self, r: Rect) {
        let r = r * self.dpi();
        self.host
            .put_bounds(unsafe {
                std::mem::transmute(RECT {
                    left: r.origin.x as _,
                    top: r.origin.y as _,
                    right: (r.origin.x + r.size.width) as _,
                    bottom: (r.origin.y + r.size.height) as _,
                })
            })
            .unwrap();
    }
}

impl WebViewImpl for WebViewInner {
    async fn new(parent: impl AsWindow) -> Self {
        let (tx, rx) = futures_channel::oneshot::channel();
        let hwnd = parent.as_window().as_win32();
        // Check here because the creation is asynchronous.
        if unsafe { IsWindow(hwnd) } == 0 {
            panic!("Invalid window handle");
        }
        Environment::builder()
            .build(move |res| {
                res?.create_controller(hwnd.cast(), |res| {
                    let host = res?;
                    let view = host.get_webview()?;
                    tx.send((host, view)).ok();
                    Ok(())
                })
            })
            .unwrap();
        let (host, view) = rx.await.unwrap();
        let (tx, rx) = flume::unbounded();
        view.add_navigation_completed(move |_, _| {
            tx.send(()).ok();
            Ok(())
        })
        .unwrap();
        Self {
            host,
            view,
            nav_rx: rx,
        }
    }

    fn is_visible(&self) -> bool {
        self.host.get_is_visible().unwrap()
    }

    fn set_visible(&mut self, v: bool) {
        self.host.put_is_visible(v).unwrap();
    }

    fn is_enabled(&self) -> bool {
        true
    }

    fn set_enabled(&mut self, _: bool) {}

    fn loc(&self) -> Point {
        self.rect().origin
    }

    fn set_loc(&mut self, p: Point) {
        let mut rect = self.rect();
        rect.origin = p;
        self.set_rect(rect);
    }

    fn size(&self) -> Size {
        self.rect().size
    }

    fn set_size(&mut self, v: Size) {
        let mut rect = self.rect();
        rect.size = v;
        self.set_rect(rect);
    }

    fn source(&self) -> String {
        self.view.get_source().unwrap()
    }

    fn set_source(&mut self, s: impl AsRef<str>) {
        self.view.navigate(s.as_ref()).unwrap();
    }

    fn can_go_forward(&self) -> bool {
        self.view.get_can_go_forward().unwrap()
    }

    fn go_forward(&mut self) {
        self.view.go_forward().unwrap();
    }

    fn can_go_back(&self) -> bool {
        self.view.get_can_go_back().unwrap()
    }

    fn go_back(&mut self) {
        self.view.go_back().unwrap();
    }

    fn wait_navigate(&self) -> impl Future<Output = ()> + 'static + use<> {
        let rx = self.nav_rx.clone();
        async move {
            rx.recv_async().await.ok();
        }
    }
}

impl AsRawWidget for WebViewInner {
    fn as_raw_widget(&self) -> RawWidget {
        unimplemented!("cannot get HWND from WebView2")
    }
}

pub type WebView = WebViewLazy<WebViewInner>;
