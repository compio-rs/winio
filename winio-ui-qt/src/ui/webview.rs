use inherit_methods_macro::inherit_methods;
use winio_callback::Callback;
use winio_handle::AsContainer;
use winio_primitive::{Point, Size};

use crate::{
    GlobalRuntime, Result,
    ui::{Widget, impl_static_cast},
};

#[derive(Debug)]
pub struct WebView {
    on_started: Box<Callback>,
    on_loaded: Box<Callback>,
    widget: Widget<ffi::QWebEngineView>,
}

#[inherit_methods(from = "self.widget")]
impl WebView {
    pub fn new(parent: impl AsContainer) -> Result<Self> {
        let mut widget = unsafe { ffi::new_webview(parent.as_container().as_qt()) }?;
        let on_started = Box::new(Callback::new());
        unsafe {
            ffi::webview_connect_load_started(
                widget.pin_mut(),
                Self::on_started,
                on_started.as_ref() as *const _ as _,
            )?;
        }
        let on_loaded = Box::new(Callback::new());
        unsafe {
            ffi::webview_connect_load_finished(
                widget.pin_mut(),
                Self::on_loaded,
                on_loaded.as_ref() as *const _ as _,
            )?;
        }
        let mut widget = Widget::new(widget)?;
        widget.set_visible(true)?;
        Ok(Self {
            on_started,
            on_loaded,
            widget,
        })
    }

    pub fn is_visible(&self) -> Result<bool>;

    pub fn set_visible(&mut self, v: bool) -> Result<()>;

    pub fn is_enabled(&self) -> Result<bool>;

    pub fn set_enabled(&mut self, v: bool) -> Result<()>;

    pub fn preferred_size(&self) -> Result<Size> {
        Ok(Size::zero())
    }

    pub fn loc(&self) -> Result<Point>;

    pub fn set_loc(&mut self, p: Point) -> Result<()>;

    pub fn size(&self) -> Result<Size>;

    pub fn set_size(&mut self, s: Size) -> Result<()>;

    pub fn source(&self) -> Result<String> {
        Ok(self.widget.as_ref().url()?.try_into()?)
    }

    pub fn set_source(&mut self, s: impl AsRef<str>) -> Result<()> {
        self.widget.pin_mut().setUrl(&s.as_ref().try_into()?)?;
        Ok(())
    }

    pub fn set_html(&mut self, html: impl AsRef<str>) -> Result<()> {
        self.widget
            .pin_mut()
            .setHtml(&html.as_ref().try_into()?, &"".try_into()?)?;
        Ok(())
    }

    pub fn can_go_forward(&self) -> Result<bool> {
        unsafe {
            if let Some(history) = self.widget.as_ref().history()?.as_ref() {
                Ok(history.canGoForward()?)
            } else {
                Ok(false)
            }
        }
    }

    pub fn go_forward(&mut self) -> Result<()> {
        self.widget.pin_mut().forward()?;
        Ok(())
    }

    pub fn can_go_back(&self) -> Result<bool> {
        unsafe {
            if let Some(history) = self.widget.as_ref().history()?.as_ref() {
                Ok(history.canGoBack()?)
            } else {
                Ok(false)
            }
        }
    }

    pub fn go_back(&mut self) -> Result<()> {
        self.widget.pin_mut().back()?;
        Ok(())
    }

    pub fn reload(&mut self) -> Result<()> {
        self.widget.pin_mut().reload()?;
        Ok(())
    }

    pub fn stop(&mut self) -> Result<()> {
        self.widget.pin_mut().stop()?;
        Ok(())
    }

    fn on_started(c: *const u8) {
        let c = c as *const Callback<()>;
        if let Some(c) = unsafe { c.as_ref() } {
            c.signal::<GlobalRuntime>(());
        }
    }

    pub async fn wait_navigating(&self) {
        self.on_started.wait().await;
    }

    fn on_loaded(c: *const u8) {
        let c = c as *const Callback<()>;
        if let Some(c) = unsafe { c.as_ref() } {
            c.signal::<GlobalRuntime>(());
        }
    }

    pub async fn wait_navigated(&self) {
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
        type QString = crate::ui::QString;
        type QWebEngineView;
        type QWebEngineHistory;

        unsafe fn new_webview(parent: *mut QWidget) -> Result<UniquePtr<QWebEngineView>>;

        unsafe fn webview_connect_load_started(
            w: Pin<&mut QWebEngineView>,
            callback: unsafe fn(*const u8),
            data: *const u8,
        ) -> Result<()>;

        unsafe fn webview_connect_load_finished(
            w: Pin<&mut QWebEngineView>,
            callback: unsafe fn(*const u8),
            data: *const u8,
        ) -> Result<()>;

        fn url(self: &QWebEngineView) -> Result<QUrl>;
        fn setUrl(self: Pin<&mut QWebEngineView>, url: &QUrl) -> Result<()>;
        fn setHtml(self: Pin<&mut QWebEngineView>, html: &QString, url: &QUrl) -> Result<()>;
        fn forward(self: Pin<&mut QWebEngineView>) -> Result<()>;
        fn back(self: Pin<&mut QWebEngineView>) -> Result<()>;
        fn history(self: &QWebEngineView) -> Result<*mut QWebEngineHistory>;
        fn reload(self: Pin<&mut QWebEngineView>) -> Result<()>;
        fn stop(self: Pin<&mut QWebEngineView>) -> Result<()>;

        fn canGoForward(self: &QWebEngineHistory) -> Result<bool>;
        fn canGoBack(self: &QWebEngineHistory) -> Result<bool>;
    }
}
