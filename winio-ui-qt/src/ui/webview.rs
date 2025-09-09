use inherit_methods_macro::inherit_methods;
use winio_callback::Callback;
use winio_handle::AsWindow;
use winio_primitive::{Point, Size};

use crate::{
    GlobalRuntime,
    ui::{Widget, impl_static_cast},
};

#[derive(Debug)]
pub struct WebView {
    on_loaded: Box<Callback>,
    widget: Widget<ffi::QWebEngineView>,
}

#[inherit_methods(from = "self.widget")]
impl WebView {
    pub fn new(parent: impl AsWindow) -> Self {
        let on_loaded = Box::new(Callback::new());
        let mut widget = unsafe { ffi::new_webview(parent.as_window().as_qt()) };
        unsafe {
            ffi::webview_connect_load_finished(
                widget.pin_mut(),
                Self::on_loaded,
                on_loaded.as_ref() as *const _ as _,
            );
        }
        let mut widget = Widget::new(widget);
        widget.set_visible(true);
        Self { on_loaded, widget }
    }

    pub fn is_visible(&self) -> bool;

    pub fn set_visible(&mut self, v: bool);

    pub fn is_enabled(&self) -> bool;

    pub fn set_enabled(&mut self, v: bool);

    pub fn preferred_size(&self) -> Size {
        Size::zero()
    }

    pub fn loc(&self) -> Point;

    pub fn set_loc(&mut self, p: Point);

    pub fn size(&self) -> Size;

    pub fn set_size(&mut self, s: Size);

    pub fn source(&self) -> String {
        self.widget.as_ref().url().into()
    }

    pub fn set_source(&mut self, s: impl AsRef<str>) {
        self.widget.pin_mut().setUrl(&s.as_ref().into());
    }

    pub fn can_go_forward(&self) -> bool {
        unsafe {
            self.widget
                .as_ref()
                .history()
                .as_ref()
                .map(|history| history.canGoForward())
                .unwrap_or_default()
        }
    }

    pub fn go_forward(&mut self) {
        self.widget.pin_mut().forward();
    }

    pub fn can_go_back(&self) -> bool {
        unsafe {
            self.widget
                .as_ref()
                .history()
                .as_ref()
                .map(|history| history.canGoBack())
                .unwrap_or_default()
        }
    }

    pub fn go_back(&mut self) {
        self.widget.pin_mut().back();
    }

    fn on_loaded(c: *const u8) {
        let c = c as *const Callback<()>;
        if let Some(c) = unsafe { c.as_ref() } {
            c.signal::<GlobalRuntime>(());
        }
    }

    pub async fn wait_navigate(&self) {
        self.on_loaded.wait().await;
    }
}

winio_handle::impl_as_widget!(WebView, widget);

impl_static_cast!(ffi::QWebEngineView, ffi::QWidget);

#[cxx::bridge]
mod ffi {
    unsafe extern "C++-unwind" {
        include!("winio-ui-qt/src/ui/webview.hpp");

        type QWidget = crate::ui::QWidget;
        type QUrl = crate::ui::QUrl;
        type QWebEngineView;
        type QWebEngineHistory;

        unsafe fn new_webview(parent: *mut QWidget) -> UniquePtr<QWebEngineView>;

        unsafe fn webview_connect_load_finished(
            w: Pin<&mut QWebEngineView>,
            callback: unsafe fn(*const u8),
            data: *const u8,
        );

        fn url(self: &QWebEngineView) -> QUrl;
        fn setUrl(self: Pin<&mut QWebEngineView>, url: &QUrl);
        fn forward(self: Pin<&mut QWebEngineView>);
        fn back(self: Pin<&mut QWebEngineView>);
        fn history(self: &QWebEngineView) -> *mut QWebEngineHistory;

        fn canGoForward(self: &QWebEngineHistory) -> bool;
        fn canGoBack(self: &QWebEngineHistory) -> bool;
    }
}
