use flume::Receiver;
use webview2::{Controller, Environment, WebView as WebView2};
use windows_sys::Win32::{Foundation::RECT, UI::HiDpi::GetDpiForWindow};
use winio_handle::{AsRawWidget, AsWidget, AsWindow, BorrowedWidget, RawWidget};
use winio_primitive::{Point, Rect, Size};

#[derive(Debug)]
pub struct WebView {
    host: Controller,
    view: WebView2,
    nav_rx: Receiver<()>,
}

impl WebView {
    pub async fn new(parent: impl AsWindow) -> Self {
        let (tx, rx) = futures_channel::oneshot::channel();
        let hwnd = parent.as_window().as_win32();
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

    pub fn is_visible(&self) -> bool {
        self.host.get_is_visible().unwrap()
    }

    pub fn set_visible(&mut self, v: bool) {
        self.host.put_is_visible(v).unwrap();
    }

    pub fn is_enabled(&self) -> bool {
        true
    }

    pub fn set_enabled(&mut self, _: bool) {}

    pub fn preferred_size(&self) -> Size {
        Size::zero()
    }

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

    pub fn loc(&self) -> Point {
        self.rect().origin
    }

    pub fn set_loc(&mut self, p: Point) {
        let mut rect = self.rect();
        rect.origin = p;
        self.set_rect(rect);
    }

    pub fn size(&self) -> Size {
        self.rect().size
    }

    pub fn set_size(&mut self, v: Size) {
        let mut rect = self.rect();
        rect.size = v;
        self.set_rect(rect);
    }

    pub fn source(&self) -> String {
        self.view.get_source().unwrap()
    }

    pub fn set_source(&self, s: impl AsRef<str>) {
        self.view.navigate(s.as_ref()).unwrap();
    }

    pub fn can_go_forward(&self) -> bool {
        self.view.get_can_go_forward().unwrap()
    }

    pub fn go_forward(&mut self) {
        self.view.go_forward().unwrap();
    }

    pub fn can_go_back(&self) -> bool {
        self.view.get_can_go_back().unwrap()
    }

    pub fn go_back(&mut self) {
        self.view.go_back().unwrap();
    }

    pub async fn wait_navigate(&self) {
        self.nav_rx.recv_async().await.ok();
    }
}

impl AsRawWidget for WebView {
    fn as_raw_widget(&self) -> RawWidget {
        unimplemented!("cannot get HWND from WebView2")
    }
}

impl AsWidget for WebView {
    fn as_widget(&self) -> BorrowedWidget<'_> {
        unsafe { BorrowedWidget::borrow_raw(AsRawWidget::as_raw_widget(self)) }
    }
}
