use std::{cell::RefCell, rc::Rc};

use cookie::Cookie;
use webview2::{
    COREWEBVIEW2_COOKIE_SAME_SITE_KIND, COREWEBVIEW2_COOKIE_SAME_SITE_KIND_LAX,
    COREWEBVIEW2_COOKIE_SAME_SITE_KIND_NONE, COREWEBVIEW2_COOKIE_SAME_SITE_KIND_STRICT,
    CreateCoreWebView2Environment, ICoreWebView2, ICoreWebView2_2, ICoreWebView2Controller,
    ICoreWebView2Cookie, ICoreWebView2Cookie_Impl, ICoreWebView2CookieList,
    ICoreWebView2CookieManager, ICoreWebView2CreateCoreWebView2ControllerCompletedHandler,
    ICoreWebView2CreateCoreWebView2ControllerCompletedHandler_Impl,
    ICoreWebView2CreateCoreWebView2EnvironmentCompletedHandler,
    ICoreWebView2CreateCoreWebView2EnvironmentCompletedHandler_Impl, ICoreWebView2Environment,
    ICoreWebView2ExecuteScriptCompletedHandler, ICoreWebView2ExecuteScriptCompletedHandler_Impl,
    ICoreWebView2GetCookiesCompletedHandler, ICoreWebView2GetCookiesCompletedHandler_Impl,
    ICoreWebView2NavigationCompletedEventArgs, ICoreWebView2NavigationCompletedEventHandler,
    ICoreWebView2NavigationCompletedEventHandler_Impl, ICoreWebView2NavigationStartingEventArgs,
    ICoreWebView2NavigationStartingEventHandler, ICoreWebView2NavigationStartingEventHandler_Impl,
};
use windows::{
    Win32::{
        Foundation::{E_FAIL, E_INVALIDARG, HWND, RECT},
        System::Com::CoTaskMemAlloc,
    },
    core::{BOOL, HRESULT, Interface, PCWSTR, PWSTR, Ref, implement},
};
use windows_sys::Win32::{Foundation::ERROR_CANCELLED, UI::HiDpi::GetDpiForWindow};
use winio_callback::Callback;
use winio_handle::{AsContainer, AsWidget, BorrowedWidget};
use winio_primitive::{Point, Rect, Size};
use winio_ui_windows_common::CoTaskMemPtr;

use crate::{Error, Result, ui::with_u16c};

#[derive(Debug)]
pub struct WebView {
    host: ICoreWebView2Controller,
    view: ICoreWebView2,
    navigating: Rc<Callback>,
    navigated: Rc<Callback>,
}

impl WebView {
    pub async fn new(parent: impl AsContainer) -> Result<Self> {
        let (tx, rx) = local_sync::oneshot::channel();
        let hwnd = parent.as_container().as_win32();
        unsafe {
            CreateCoreWebView2Environment(&CreateEnvHandler::create(move |env| {
                let env = env?;
                let env = env.ok()?;
                env.CreateCoreWebView2Controller(
                    HWND(hwnd),
                    &CreateControllerHandler::create(move |host| {
                        let host = host?;
                        let host = host.ok()?;
                        let view = host.CoreWebView2()?;
                        tx.send((host.clone(), view)).ok();
                        Ok(())
                    }),
                )?;
                Ok(())
            }))?;
        }
        let (host, view) = rx.await.map_err(|_| Error::from_hresult(E_FAIL))?;
        let navigating = Rc::new(Callback::new());
        unsafe {
            let navigating = navigating.clone();
            view.NavigationStarting(&NavStartingHandler::create(move |_, _| {
                navigating.signal::<()>(());
                Ok(())
            }))?;
        }
        let navigated = Rc::new(Callback::new());
        unsafe {
            let navigated = navigated.clone();
            view.NavigationCompleted(&NavCompletedHandler::create(move |_, _| {
                navigated.signal::<()>(());
                Ok(())
            }))?;
        }
        unsafe {
            host.SetIsVisible(true)?;
        }
        Ok(Self {
            host,
            view,
            navigating,
            navigated,
        })
    }

    fn dpi(&self) -> Result<f64> {
        unsafe {
            let hwnd = self.host.ParentWindow()?;
            Ok(GetDpiForWindow(hwnd.0) as f64 / 96.0)
        }
    }

    fn rect(&self) -> Result<Rect> {
        let rect = unsafe { self.host.Bounds() }?;
        Ok(Rect::new(
            Point::new(rect.left as _, rect.top as _),
            Size::new((rect.right - rect.left) as _, (rect.bottom - rect.top) as _),
        ) / self.dpi()?)
    }

    fn set_rect(&mut self, r: Rect) -> Result<()> {
        let r = r * self.dpi()?;
        unsafe {
            self.host.SetBounds(RECT {
                left: r.origin.x as _,
                top: r.origin.y as _,
                right: (r.origin.x + r.size.width) as _,
                bottom: (r.origin.y + r.size.height) as _,
            })?;
        }
        Ok(())
    }

    pub fn is_visible(&self) -> Result<bool> {
        unsafe { Ok(self.host.IsVisible()?.as_bool()) }
    }

    pub fn set_visible(&mut self, v: bool) -> Result<()> {
        unsafe {
            self.host.SetIsVisible(v)?;
            Ok(())
        }
    }

    pub fn is_enabled(&self) -> Result<bool> {
        Ok(true)
    }

    pub fn set_enabled(&mut self, _: bool) -> Result<()> {
        Ok(())
    }

    pub fn loc(&self) -> Result<Point> {
        Ok(self.rect()?.origin)
    }

    pub fn set_loc(&mut self, p: Point) -> Result<()> {
        let mut rect = self.rect()?;
        rect.origin = p;
        self.set_rect(rect)
    }

    pub fn size(&self) -> Result<Size> {
        Ok(self.rect()?.size)
    }

    pub fn set_size(&mut self, v: Size) -> Result<()> {
        let mut rect = self.rect()?;
        rect.size = v;
        self.set_rect(rect)
    }

    pub fn source(&self) -> Result<String> {
        unsafe {
            let source = CoTaskMemPtr::new(self.view.Source()?.0);
            source.to_string()
        }
    }

    pub fn set_source(&mut self, s: impl AsRef<str>) -> Result<()> {
        let s = s.as_ref();
        if s.is_empty() {
            return self.set_html("");
        }
        with_u16c(s, |s| unsafe {
            self.view.Navigate(PCWSTR(s.as_ptr()))?;
            Ok(())
        })
    }

    pub fn set_html(&mut self, s: impl AsRef<str>) -> Result<()> {
        with_u16c(s.as_ref(), |s| unsafe {
            self.view.NavigateToString(PCWSTR(s.as_ptr()))?;
            Ok(())
        })
    }

    pub fn can_go_forward(&self) -> Result<bool> {
        unsafe { Ok(self.view.CanGoForward()?.as_bool()) }
    }

    pub fn go_forward(&mut self) -> Result<()> {
        unsafe {
            self.view.GoForward()?;
            Ok(())
        }
    }

    pub fn can_go_back(&self) -> Result<bool> {
        unsafe { Ok(self.view.CanGoBack()?.as_bool()) }
    }

    pub fn go_back(&mut self) -> Result<()> {
        unsafe {
            self.view.GoBack()?;
            Ok(())
        }
    }

    pub fn reload(&mut self) -> Result<()> {
        unsafe {
            self.view.Reload()?;
            Ok(())
        }
    }

    pub fn stop(&mut self) -> Result<()> {
        unsafe {
            self.view.Stop()?;
            Ok(())
        }
    }

    pub async fn wait_navigating(&self) {
        self.navigating.wait().await;
    }

    pub async fn wait_navigated(&self) {
        self.navigated.wait().await;
    }

    fn cookie_manager(&self) -> Result<ICoreWebView2CookieManager> {
        unsafe { self.view.cast::<ICoreWebView2_2>()?.CookieManager() }
    }

    pub async fn cookies(&self) -> Result<Vec<Cookie<'static>>> {
        let (tx, rx) = local_sync::oneshot::channel();
        let handler = GetCookiesHandler::create(move |result| {
            fn conv_cookies(cookies: Ref<ICoreWebView2CookieList>) -> Result<Vec<Cookie<'static>>> {
                let list = cookies.ok()?;
                let mut cookies = vec![];
                for i in 0..unsafe { list.Count()? } {
                    let cookie = unsafe { list.GetValueAtIndex(i)? };
                    cookies.push(webview_cookie_to_cookie(&cookie)?);
                }
                Ok(cookies)
            }
            tx.send(result.map(conv_cookies)).ok();
            Ok(())
        });
        unsafe { self.cookie_manager()?.GetCookies(None, &handler)? };
        rx.await
            .map_err(|_| Error::from_hresult(HRESULT::from_win32(ERROR_CANCELLED)))??
    }

    pub async fn set_cookie(&mut self, c: &Cookie<'_>) -> Result<()> {
        unsafe {
            self.cookie_manager()?
                .AddOrUpdateCookie(&cookie_to_webview_cookie(c))?;
        }
        Ok(())
    }

    pub async fn delete_cookie(&mut self, c: &Cookie<'_>) -> Result<()> {
        unsafe {
            self.cookie_manager()?
                .DeleteCookie(&cookie_to_webview_cookie(c))?;
        }
        Ok(())
    }

    pub async fn run_javascript(&mut self, s: impl AsRef<str>) -> Result<String> {
        let s = s.as_ref();
        let (tx, rx) = local_sync::oneshot::channel();
        with_u16c(s, |s| unsafe {
            self.view.ExecuteScript(
                PCWSTR(s.as_ptr()),
                &ExecuteScriptHandler::create(move |result| {
                    tx.send(result.map(|s| s.to_hstring())).ok();
                    Ok(())
                }),
            )?;
            Ok(())
        })?;
        let result = rx
            .await
            .map_err(|_| Error::from_hresult(HRESULT::from_win32(ERROR_CANCELLED)))??;
        Ok(result.to_string_lossy())
    }
}

impl AsWidget for WebView {
    fn as_widget(&self) -> BorrowedWidget<'_> {
        unimplemented!("cannot get HWND from WebView2")
    }
}

fn cookie_to_webview_cookie(c: &Cookie<'_>) -> ICoreWebView2Cookie {
    CookieWrap::create(c.clone().into_owned())
}

fn webview_cookie_to_cookie(c: &ICoreWebView2Cookie) -> Result<Cookie<'static>> {
    let name = unsafe { CoTaskMemPtr::new(c.Name()?.0) };
    let value = unsafe { CoTaskMemPtr::new(c.Value()?.0) };
    let domain = unsafe { CoTaskMemPtr::new(c.Domain()?.0) };
    let path = unsafe { CoTaskMemPtr::new(c.Path()?.0) };
    let expires = unsafe { c.Expires() }?;
    let is_secure = unsafe { c.IsSecure()?.as_bool() };
    let is_http_only = unsafe { c.IsHttpOnly()?.as_bool() };
    let same_site = unsafe { c.SameSite()? };
    let is_session = unsafe { c.IsSession()?.as_bool() };
    let cookie = Cookie::build((unsafe { name.to_string()? }, unsafe { value.to_string()? }))
        .domain(unsafe { domain.to_string()? })
        .path(unsafe { path.to_string()? })
        .expires(if is_session {
            cookie::Expiration::Session
        } else {
            cookie::Expiration::DateTime(
                time::OffsetDateTime::from_unix_timestamp(expires as _)
                    .map_err(|_| Error::from_hresult(E_INVALIDARG))?,
            )
        })
        .secure(is_secure)
        .http_only(is_http_only)
        .same_site(match same_site {
            COREWEBVIEW2_COOKIE_SAME_SITE_KIND_LAX => cookie::SameSite::Lax,
            COREWEBVIEW2_COOKIE_SAME_SITE_KIND_STRICT => cookie::SameSite::Strict,
            COREWEBVIEW2_COOKIE_SAME_SITE_KIND_NONE => cookie::SameSite::None,
            _ => return Err(Error::from_hresult(E_INVALIDARG)),
        })
        .build();
    Ok(cookie)
}

#[implement(
    ICoreWebView2CreateCoreWebView2EnvironmentCompletedHandler,
    Agile = false
)]
struct CreateEnvHandler<F>
where
    F: FnOnce(Result<Ref<ICoreWebView2Environment>>) -> Result<()> + 'static,
{
    f: RefCell<Option<F>>,
}

impl<F> CreateEnvHandler<F>
where
    F: FnOnce(Result<Ref<ICoreWebView2Environment>>) -> Result<()> + 'static,
{
    pub fn create(f: F) -> ICoreWebView2CreateCoreWebView2EnvironmentCompletedHandler {
        Self {
            f: RefCell::new(Some(f)),
        }
        .into()
    }
}

impl<F> ICoreWebView2CreateCoreWebView2EnvironmentCompletedHandler_Impl for CreateEnvHandler_Impl<F>
where
    F: FnOnce(Result<Ref<ICoreWebView2Environment>>) -> Result<()> + 'static,
{
    fn Invoke(
        &self,
        errorcode: HRESULT,
        createdenvironment: Ref<ICoreWebView2Environment>,
    ) -> Result<()> {
        let f = self.f.borrow_mut().take();
        if let Some(f) = f {
            f(errorcode.map(|| createdenvironment))
        } else {
            Ok(())
        }
    }
}

#[implement(
    ICoreWebView2CreateCoreWebView2ControllerCompletedHandler,
    Agile = false
)]
struct CreateControllerHandler<F>
where
    F: FnOnce(Result<Ref<ICoreWebView2Controller>>) -> Result<()> + 'static,
{
    f: RefCell<Option<F>>,
}

impl<F> CreateControllerHandler<F>
where
    F: FnOnce(Result<Ref<ICoreWebView2Controller>>) -> Result<()> + 'static,
{
    pub fn create(f: F) -> ICoreWebView2CreateCoreWebView2ControllerCompletedHandler {
        Self {
            f: RefCell::new(Some(f)),
        }
        .into()
    }
}

impl<F> ICoreWebView2CreateCoreWebView2ControllerCompletedHandler_Impl
    for CreateControllerHandler_Impl<F>
where
    F: FnOnce(Result<Ref<ICoreWebView2Controller>>) -> Result<()> + 'static,
{
    fn Invoke(
        &self,
        errorcode: HRESULT,
        createdcontroller: Ref<ICoreWebView2Controller>,
    ) -> Result<()> {
        let f = self.f.borrow_mut().take();
        if let Some(f) = f {
            f(errorcode.map(|| createdcontroller))
        } else {
            Ok(())
        }
    }
}

#[implement(ICoreWebView2NavigationStartingEventHandler, Agile = false)]
struct NavStartingHandler<F>
where
    F: Fn(Ref<ICoreWebView2>, Ref<ICoreWebView2NavigationStartingEventArgs>) -> Result<()>
        + 'static,
{
    f: F,
}

impl<F> NavStartingHandler<F>
where
    F: Fn(Ref<ICoreWebView2>, Ref<ICoreWebView2NavigationStartingEventArgs>) -> Result<()>
        + 'static,
{
    pub fn create(f: F) -> ICoreWebView2NavigationStartingEventHandler {
        Self { f }.into()
    }
}

impl<F> ICoreWebView2NavigationStartingEventHandler_Impl for NavStartingHandler_Impl<F>
where
    F: Fn(Ref<ICoreWebView2>, Ref<ICoreWebView2NavigationStartingEventArgs>) -> Result<()>
        + 'static,
{
    fn Invoke(
        &self,
        sender: Ref<ICoreWebView2>,
        args: Ref<ICoreWebView2NavigationStartingEventArgs>,
    ) -> Result<()> {
        (self.f)(sender, args)
    }
}

#[implement(ICoreWebView2NavigationCompletedEventHandler, Agile = false)]
struct NavCompletedHandler<F>
where
    F: Fn(Ref<ICoreWebView2>, Ref<ICoreWebView2NavigationCompletedEventArgs>) -> Result<()>
        + 'static,
{
    f: F,
}

impl<F> NavCompletedHandler<F>
where
    F: Fn(Ref<ICoreWebView2>, Ref<ICoreWebView2NavigationCompletedEventArgs>) -> Result<()>
        + 'static,
{
    pub fn create(f: F) -> ICoreWebView2NavigationCompletedEventHandler {
        Self { f }.into()
    }
}

impl<F> ICoreWebView2NavigationCompletedEventHandler_Impl for NavCompletedHandler_Impl<F>
where
    F: Fn(Ref<ICoreWebView2>, Ref<ICoreWebView2NavigationCompletedEventArgs>) -> Result<()>
        + 'static,
{
    fn Invoke(
        &self,
        sender: Ref<ICoreWebView2>,
        args: Ref<ICoreWebView2NavigationCompletedEventArgs>,
    ) -> Result<()> {
        (self.f)(sender, args)
    }
}

#[implement(ICoreWebView2GetCookiesCompletedHandler, Agile = false)]
struct GetCookiesHandler<F>
where
    F: FnOnce(Result<Ref<ICoreWebView2CookieList>>) -> Result<()> + 'static,
{
    f: RefCell<Option<F>>,
}

impl<F> GetCookiesHandler<F>
where
    F: FnOnce(Result<Ref<ICoreWebView2CookieList>>) -> Result<()> + 'static,
{
    pub fn create(f: F) -> ICoreWebView2GetCookiesCompletedHandler {
        Self {
            f: RefCell::new(Some(f)),
        }
        .into()
    }
}

impl<F> ICoreWebView2GetCookiesCompletedHandler_Impl for GetCookiesHandler_Impl<F>
where
    F: FnOnce(Result<Ref<ICoreWebView2CookieList>>) -> Result<()> + 'static,
{
    fn Invoke(&self, errorcode: HRESULT, cookie_list: Ref<ICoreWebView2CookieList>) -> Result<()> {
        let f = self.f.borrow_mut().take();
        if let Some(f) = f {
            f(errorcode.map(|| cookie_list))
        } else {
            Ok(())
        }
    }
}

#[implement(ICoreWebView2Cookie, Agile = false)]
struct CookieWrap(RefCell<Cookie<'static>>);

impl CookieWrap {
    pub fn create(c: Cookie<'static>) -> ICoreWebView2Cookie {
        Self(RefCell::new(c)).into()
    }
}

fn create_pwstr(s: &str) -> PWSTR {
    let v: Vec<u16> = s.encode_utf16().collect();
    unsafe {
        let ptr = CoTaskMemAlloc(v.len() * 2 + 2).cast::<u16>();
        std::ptr::copy_nonoverlapping(v.as_ptr(), ptr, v.len());
        *ptr.add(v.len()) = 0;
        PWSTR(ptr)
    }
}

impl ICoreWebView2Cookie_Impl for CookieWrap_Impl {
    fn Name(&self) -> Result<PWSTR> {
        Ok(create_pwstr(self.0.borrow().name()))
    }

    fn Value(&self) -> Result<PWSTR> {
        Ok(create_pwstr(self.0.borrow().value()))
    }

    fn SetValue(&self, value: &PCWSTR) -> Result<()> {
        self.0.borrow_mut().set_value(unsafe { value.to_string()? });
        Ok(())
    }

    fn Domain(&self) -> Result<PWSTR> {
        Ok(create_pwstr(self.0.borrow().domain().unwrap_or_default()))
    }

    fn Path(&self) -> Result<PWSTR> {
        Ok(create_pwstr(self.0.borrow().path().unwrap_or_default()))
    }

    fn Expires(&self) -> Result<f64> {
        let expires = self.0.borrow().expires();
        Ok(match expires {
            Some(cookie::Expiration::DateTime(dt)) => dt.unix_timestamp() as f64,
            _ => -1.0,
        })
    }

    fn SetExpires(&self, expires: f64) -> Result<()> {
        let expires = if expires < 0.0 {
            cookie::Expiration::Session
        } else {
            cookie::Expiration::DateTime(
                time::OffsetDateTime::from_unix_timestamp(expires as _)
                    .map_err(|_| Error::from_hresult(E_INVALIDARG))?,
            )
        };
        self.0.borrow_mut().set_expires(expires);
        Ok(())
    }

    fn IsHttpOnly(&self) -> Result<BOOL> {
        Ok(self.0.borrow().http_only().unwrap_or_default().into())
    }

    fn SetIsHttpOnly(&self, ishttponly: BOOL) -> Result<()> {
        self.0.borrow_mut().set_http_only(ishttponly.as_bool());
        Ok(())
    }

    fn SameSite(&self) -> Result<COREWEBVIEW2_COOKIE_SAME_SITE_KIND> {
        Ok(match self.0.borrow().same_site() {
            Some(cookie::SameSite::Strict) => COREWEBVIEW2_COOKIE_SAME_SITE_KIND_STRICT,
            Some(cookie::SameSite::None) => COREWEBVIEW2_COOKIE_SAME_SITE_KIND_NONE,
            _ => COREWEBVIEW2_COOKIE_SAME_SITE_KIND_LAX,
        })
    }

    fn SetSameSite(&self, samesite: COREWEBVIEW2_COOKIE_SAME_SITE_KIND) -> Result<()> {
        let same_site = match samesite {
            COREWEBVIEW2_COOKIE_SAME_SITE_KIND_STRICT => cookie::SameSite::Strict,
            COREWEBVIEW2_COOKIE_SAME_SITE_KIND_NONE => cookie::SameSite::None,
            _ => cookie::SameSite::Lax,
        };
        self.0.borrow_mut().set_same_site(Some(same_site));
        Ok(())
    }

    fn IsSecure(&self) -> Result<BOOL> {
        Ok(self.0.borrow().secure().unwrap_or_default().into())
    }

    fn SetIsSecure(&self, issecure: BOOL) -> Result<()> {
        self.0.borrow_mut().set_secure(issecure.as_bool());
        Ok(())
    }

    fn IsSession(&self) -> Result<BOOL> {
        Ok(self
            .0
            .borrow()
            .expires()
            .is_some_and(|e| matches!(e, cookie::Expiration::Session))
            .into())
    }
}

#[implement(ICoreWebView2ExecuteScriptCompletedHandler, Agile = false)]
struct ExecuteScriptHandler<F>
where
    F: FnOnce(Result<PCWSTR>) -> Result<()> + 'static,
{
    f: RefCell<Option<F>>,
}

impl<F> ExecuteScriptHandler<F>
where
    F: FnOnce(Result<PCWSTR>) -> Result<()> + 'static,
{
    pub fn create(f: F) -> ICoreWebView2ExecuteScriptCompletedHandler {
        Self {
            f: RefCell::new(Some(f)),
        }
        .into()
    }
}

impl<F> ICoreWebView2ExecuteScriptCompletedHandler_Impl for ExecuteScriptHandler_Impl<F>
where
    F: FnOnce(Result<PCWSTR>) -> Result<()> + 'static,
{
    fn Invoke(&self, errorcode: HRESULT, resultobjectasjson: &PCWSTR) -> Result<()> {
        let f = self.f.borrow_mut().take();
        if let Some(f) = f {
            f(errorcode.map(|| *resultobjectasjson))
        } else {
            Ok(())
        }
    }
}
