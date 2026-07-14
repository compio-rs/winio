use std::{
    cell::{RefCell, UnsafeCell},
    fmt::Debug,
    pin::Pin,
};

use cookie::Cookie;
use cxx::{ExternType, UniquePtr, type_id};
use futures_util::TryFutureExt;
use inherit_methods_macro::inherit_methods;
use winio_callback::Callback;
use winio_handle::AsContainer;
use winio_primitive::{Point, Size};

use crate::{
    Error, GlobalRuntime, Result,
    widgets::{Widget, impl_static_cast},
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
            && let Ok(cookie) = cookie_from_qt(cookie)
        {
            c.borrow_mut().push(cookie);
        }
    }

    fn on_store_delete(c: *const u8, cookie: &ffi::QNetworkCookie) {
        let c = c as *const RefCell<Vec<Cookie<'static>>>;
        if let Some(c) = unsafe { c.as_ref() }
            && let Ok(cookie) = cookie_from_qt(cookie)
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
        let c = cookie_to_qt(c)?;
        ffi::webview_cookie_store_add(self.cookie_store()?, &c)?;
        Ok(())
    }

    pub async fn delete_cookie(&mut self, c: &Cookie<'_>) -> Result<()> {
        let c = cookie_to_qt(c)?;
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

    pub fn run_javascript(
        &mut self,
        js: impl AsRef<str>,
    ) -> Result<impl Future<Output = Result<String>> + 'static> {
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
        Ok(rx.map_err(|e| e.into()))
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

fn cookie_from_qt(cookie: &ffi::QNetworkCookie) -> Result<Cookie<'static>> {
    let name: String = cookie.name()?.try_into()?;
    let value: String = cookie.value()?.try_into()?;
    let mut builder = Cookie::build((name, value))
        .domain(String::try_from(cookie.domain()?)?)
        .path(String::try_from(cookie.path()?)?)
        .secure(cookie.isSecure()?)
        .http_only(cookie.isHttpOnly()?);
    if let Some(s) = Option::<cookie::SameSite>::from(ffi::cookie_same_site(cookie)?) {
        builder = builder.same_site(s);
    }
    if cookie.isSessionCookie()? {
        builder = builder.expires(cookie::Expiration::Session);
    } else {
        let expire = ffi::cookie_expiration(cookie)?;
        if expire > 0 {
            builder = builder.expires(cookie::Expiration::DateTime(
                time::OffsetDateTime::from_unix_timestamp(expire)?,
            ));
        }
    }
    Ok(builder.build())
}

fn cookie_to_qt(cookie: &Cookie<'_>) -> Result<UniquePtr<ffi::QNetworkCookie>> {
    let mut c = ffi::new_cookie(
        &cookie.name().as_bytes().try_into()?,
        &cookie.value().as_bytes().try_into()?,
    )?;
    if let Some(s) = cookie.domain() {
        c.pin_mut().setDomain(&s.try_into()?)?;
    }
    if let Some(s) = cookie.path() {
        c.pin_mut().setPath(&s.try_into()?)?;
    }
    if let Some(b) = cookie.secure() {
        c.pin_mut().setSecure(b)?;
    }
    if let Some(b) = cookie.http_only() {
        c.pin_mut().setHttpOnly(b)?;
    }
    ffi::cookie_set_same_site(c.pin_mut(), cookie.same_site().into())?;
    if let Some(cookie::Expiration::DateTime(dt)) = cookie.expires() {
        ffi::cookie_set_expiration(c.pin_mut(), dt.unix_timestamp())?;
    }
    Ok(c)
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
#[repr(i32)]
pub enum QNetworkCookieSameSite {
    Default,
    None,
    Lax,
    Strict,
}

unsafe impl ExternType for QNetworkCookieSameSite {
    type Id = type_id!("QNetworkCookieSameSite");
    type Kind = cxx::kind::Trivial;
}

impl From<QNetworkCookieSameSite> for Option<cookie::SameSite> {
    fn from(s: QNetworkCookieSameSite) -> Self {
        match s {
            QNetworkCookieSameSite::Default => None,
            QNetworkCookieSameSite::None => Some(cookie::SameSite::None),
            QNetworkCookieSameSite::Lax => Some(cookie::SameSite::Lax),
            QNetworkCookieSameSite::Strict => Some(cookie::SameSite::Strict),
        }
    }
}

impl From<cookie::SameSite> for QNetworkCookieSameSite {
    fn from(s: cookie::SameSite) -> Self {
        match s {
            cookie::SameSite::None => QNetworkCookieSameSite::None,
            cookie::SameSite::Lax => QNetworkCookieSameSite::Lax,
            cookie::SameSite::Strict => QNetworkCookieSameSite::Strict,
        }
    }
}

impl From<Option<cookie::SameSite>> for QNetworkCookieSameSite {
    fn from(s: Option<cookie::SameSite>) -> Self {
        match s {
            None => QNetworkCookieSameSite::Default,
            Some(v) => v.into(),
        }
    }
}

#[cxx::bridge]
mod ffi {
    unsafe extern "C++-unwind" {
        include!("winio-ui-qt/src/widgets/webview.hpp");

        type QWidget = crate::widgets::QWidget;
        type QUrl = crate::common::QUrl;
        type QString = crate::common::QString;
        type QByteArray = crate::common::QByteArray;
        type QWebEngineView;
        type QWebEngineHistory;
        type QWebEnginePage;
        type QWebEngineProfile;
        type QWebEngineCookieStore;
        type QNetworkCookie;
        type QNetworkCookieSameSite = super::QNetworkCookieSameSite;

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
            cookie: &QNetworkCookie,
        ) -> Result<()>;
        fn webview_cookie_store_delete(
            store: Pin<&mut QWebEngineCookieStore>,
            cookie: &QNetworkCookie,
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

        fn new_cookie(name: &QByteArray, value: &QByteArray) -> Result<UniquePtr<QNetworkCookie>>;

        fn name(self: &QNetworkCookie) -> Result<QByteArray>;
        fn value(self: &QNetworkCookie) -> Result<QByteArray>;
        fn domain(self: &QNetworkCookie) -> Result<QString>;
        fn setDomain(self: Pin<&mut QNetworkCookie>, domain: &QString) -> Result<()>;
        fn path(self: &QNetworkCookie) -> Result<QString>;
        fn setPath(self: Pin<&mut QNetworkCookie>, path: &QString) -> Result<()>;
        fn isSecure(self: &QNetworkCookie) -> Result<bool>;
        fn setSecure(self: Pin<&mut QNetworkCookie>, secure: bool) -> Result<()>;
        fn isHttpOnly(self: &QNetworkCookie) -> Result<bool>;
        fn setHttpOnly(self: Pin<&mut QNetworkCookie>, http_only: bool) -> Result<()>;
        fn isSessionCookie(self: &QNetworkCookie) -> Result<bool>;

        fn cookie_same_site(c: &QNetworkCookie) -> Result<QNetworkCookieSameSite>;
        fn cookie_set_same_site(
            c: Pin<&mut QNetworkCookie>,
            s: QNetworkCookieSameSite,
        ) -> Result<()>;
        fn cookie_expiration(c: &QNetworkCookie) -> Result<i64>;
        fn cookie_set_expiration(c: Pin<&mut QNetworkCookie>, expiration: i64) -> Result<()>;
    }
}
