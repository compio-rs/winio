use std::{
    cell::{RefCell, UnsafeCell},
    fmt::Debug,
    pin::Pin,
};

use cookie::Cookie;
use cxx::UniquePtr;
use inherit_methods_macro::inherit_methods;
use winio_callback::Callback;
use winio_handle::AsContainer;
use winio_primitive::{Point, Size};

use crate::{
    Error, GlobalRuntime, Result,
    ui::{Widget, impl_static_cast},
};

pub struct WebView {
    on_started: Box<Callback>,
    on_loaded: Box<Callback>,
    widget: Widget<ffi::QWebEngineView>,
    profile: UniquePtr<ffi::QWebEngineProfile>,
    cookies: Box<RefCell<Vec<Cookie<'static>>>>,
}

#[inherit_methods(from = "self.widget")]
impl WebView {
    pub async fn new(parent: impl AsContainer) -> Result<Self> {
        let mut profile = ffi::new_webview_profile()?;
        let cookies = Box::new(RefCell::new(Vec::new()));
        let mut store = unsafe {
            Pin::new_unchecked(
                profile
                    .pin_mut()
                    .cookieStore()?
                    .as_mut()
                    .ok_or(Error::NotSupported)?,
            )
        };
        unsafe {
            ffi::webview_cookie_store_connect_add(
                store.as_mut(),
                Self::on_store_add,
                cookies.as_ref() as *const _ as _,
            )?;
            ffi::webview_cookie_store_connect_delete(
                store.as_mut(),
                Self::on_store_delete,
                cookies.as_ref() as *const _ as _,
            )?;
        }

        let mut widget =
            unsafe { ffi::new_webview(profile.as_mut_ptr(), parent.as_container().as_qt()) }?;
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
            profile,
            cookies,
        })
    }

    pub fn is_visible(&self) -> Result<bool>;

    pub fn set_visible(&mut self, v: bool) -> Result<()>;

    pub fn is_enabled(&self) -> Result<bool>;

    pub fn set_enabled(&mut self, v: bool) -> Result<()>;

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

    fn on_store_add(c: *const u8, cookie: &ffi::QNetworkCookie) {
        let c = c as *const RefCell<Vec<Cookie<'static>>>;
        if let Some(c) = unsafe { c.as_ref() }
            && let Some(cookie) = to_cookie(cookie)
        {
            c.borrow_mut().push(cookie);
        }
    }

    fn on_store_delete(c: *const u8, cookie: &ffi::QNetworkCookie) {
        let c = c as *const RefCell<Vec<Cookie<'static>>>;
        if let Some(c) = unsafe { c.as_ref() }
            && let Some(cookie) = to_cookie(cookie)
        {
            c.borrow_mut().retain(|c| c != &cookie);
        }
    }

    pub async fn cookies(&self) -> Result<Vec<Cookie<'static>>> {
        Ok(self.cookies.borrow().clone())
    }

    fn cookie_store(&mut self) -> Result<Pin<&mut ffi::QWebEngineCookieStore>> {
        unsafe {
            Ok(Pin::new_unchecked(
                self.profile
                    .pin_mut()
                    .cookieStore()?
                    .as_mut()
                    .ok_or(Error::NotSupported)?,
            ))
        }
    }

    pub async fn set_cookie(&mut self, c: &Cookie<'_>) -> Result<()> {
        let c = c.to_string();
        ffi::webview_cookie_store_add(self.cookie_store()?, &c)?;
        Ok(())
    }

    pub async fn delete_cookie(&mut self, c: &Cookie<'_>) -> Result<()> {
        let c = c.to_string();
        ffi::webview_cookie_store_delete(self.cookie_store()?, &c)?;
        Ok(())
    }

    fn page(&mut self) -> Result<Pin<&mut ffi::QWebEnginePage>> {
        unsafe {
            Ok(Pin::new_unchecked(
                self.widget
                    .as_ref()
                    .page()?
                    .as_mut()
                    .ok_or(Error::NotSupported)?,
            ))
        }
    }

    pub async fn run_javascript(&mut self, js: impl AsRef<str>) -> Result<String> {
        let js = js.as_ref().try_into()?;
        let (tx, rx) = local_sync::oneshot::channel();
        let tx = UnsafeCell::new(Some(tx));
        fn on_js_result(c: *const u8, result: &ffi::QString) {
            let c = c as *const UnsafeCell<Option<local_sync::oneshot::Sender<String>>>;
            if let Some(c) = unsafe { c.as_ref() }
                && let Some(c) = unsafe { c.get().as_mut() }
                && let Some(tx) = c.take()
            {
                tx.send(result.try_into().unwrap_or_default()).ok();
            }
        }
        unsafe {
            ffi::webview_page_run_js(
                self.page()?,
                &js,
                on_js_result,
                std::ptr::addr_of!(tx).cast(),
            )?;
        }
        Ok(rx.await?)
    }
}

winio_handle::impl_as_widget!(WebView, widget);

impl Debug for WebView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WebView")
            .field("on_started", &self.on_started)
            .field("on_loaded", &self.on_loaded)
            .field("widget", &self.widget)
            .finish_non_exhaustive()
    }
}

impl_static_cast!(ffi::QWebEngineView, ffi::QWidget);

fn to_cookie(cookie: &ffi::QNetworkCookie) -> Option<Cookie<'static>> {
    Cookie::parse::<String>(ffi::cookie_to_raw(cookie).ok()?.try_into().ok()?).ok()
}

#[cxx::bridge]
mod ffi {
    unsafe extern "C++-unwind" {
        include!("winio-ui-qt/src/ui/webview.hpp");

        type QWidget = crate::ui::QWidget;
        type QUrl = crate::ui::QUrl;
        type QString = crate::ui::QString;
        type QWebEngineView;
        type QWebEngineHistory;
        type QWebEnginePage;
        type QWebEngineProfile;
        type QWebEngineCookieStore;
        type QNetworkCookie;

        unsafe fn new_webview(
            profile: *mut QWebEngineProfile,
            parent: *mut QWidget,
        ) -> Result<UniquePtr<QWebEngineView>>;

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
        fn page(self: &QWebEngineView) -> Result<*mut QWebEnginePage>;

        fn canGoForward(self: &QWebEngineHistory) -> Result<bool>;
        fn canGoBack(self: &QWebEngineHistory) -> Result<bool>;

        unsafe fn webview_page_run_js(
            p: Pin<&mut QWebEnginePage>,
            js: &QString,
            callback: unsafe fn(*const u8, &QString),
            data: *const u8,
        ) -> Result<()>;

        fn new_webview_profile() -> Result<UniquePtr<QWebEngineProfile>>;

        fn cookieStore(self: Pin<&mut QWebEngineProfile>) -> Result<*mut QWebEngineCookieStore>;

        fn webview_cookie_store_add(
            store: Pin<&mut QWebEngineCookieStore>,
            cookies: &str,
        ) -> Result<()>;
        fn webview_cookie_store_delete(
            store: Pin<&mut QWebEngineCookieStore>,
            cookies: &str,
        ) -> Result<()>;

        unsafe fn webview_cookie_store_connect_add(
            store: Pin<&mut QWebEngineCookieStore>,
            callback: unsafe fn(*const u8, &QNetworkCookie),
            data: *const u8,
        ) -> Result<()>;

        unsafe fn webview_cookie_store_connect_delete(
            store: Pin<&mut QWebEngineCookieStore>,
            callback: unsafe fn(*const u8, &QNetworkCookie),
            data: *const u8,
        ) -> Result<()>;

        fn cookie_to_raw(c: &QNetworkCookie) -> Result<QString>;
    }
}
