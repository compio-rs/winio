use std::rc::Rc;

use cookie::Cookie;
use futures_util::TryFutureExt;
use inherit_methods_macro::inherit_methods;
use send_wrapper::SendWrapper;
use windows::{
    Foundation::{TypedEventHandler, Uri},
    Win32::Foundation::E_INVALIDARG,
    core::{HSTRING, Interface, h},
};
use winio_callback::Callback;
use winio_handle::AsContainer;
use winio_primitive::{Point, Size};
use winui3::Microsoft::{
    UI::Xaml::Controls as MUXC,
    Web::WebView2::Core::{
        CoreWebView2Cookie, CoreWebView2CookieManager, CoreWebView2CookieSameSiteKind,
    },
};

use crate::{Error, GlobalRuntime, Result, Widget};

#[derive(Debug)]
pub struct WebView {
    on_navigating: SendWrapper<Rc<Callback>>,
    on_navigated: SendWrapper<Rc<Callback>>,
    handle: Widget,
    view: MUXC::WebView2,
}

#[inherit_methods(from = "self.handle")]
impl WebView {
    pub async fn new(parent: impl AsContainer) -> Result<Self> {
        #[cfg(feature = "webview-system")]
        {
            fn add_webview2sdk_path() {
                use std::path::PathBuf;

                use windows::{
                    Win32::{
                        System::LibraryLoader::{
                            AddDllDirectory, LOAD_LIBRARY_SEARCH_SYSTEM32,
                            LOAD_LIBRARY_SEARCH_USER_DIRS, SetDefaultDllDirectories,
                        },
                        UI::Shell::{CSIDL_WINDOWS, SHGetSpecialFolderPathW},
                    },
                    core::PCWSTR,
                };

                unsafe {
                    SetDefaultDllDirectories(
                        LOAD_LIBRARY_SEARCH_USER_DIRS | LOAD_LIBRARY_SEARCH_SYSTEM32,
                    )
                    .ok();

                    let mut buffer = [0u16; 260];
                    if SHGetSpecialFolderPathW(None, &mut buffer, CSIDL_WINDOWS as _, false)
                        .ok()
                        .is_ok()
                    {
                        let windir =
                            widestring::U16CStr::from_ptr_str(buffer.as_ptr()).to_os_string();
                        let dlldir = PathBuf::from(windir).join(r"SystemApps\Shared\WebView2SDK");

                        if let Ok(dlldir) = widestring::U16CString::from_os_str(&dlldir) {
                            AddDllDirectory(PCWSTR(dlldir.as_ptr()));
                        }
                    }
                }
            }

            use std::sync::Once;

            static ADD_PATH: Once = Once::new();

            ADD_PATH.call_once(add_webview2sdk_path);
        }
        let view = MUXC::WebView2::new()?;
        view.EnsureCoreWebView2Async()?.await?;
        let on_navigating = SendWrapper::new(Rc::new(Callback::new()));
        {
            let on_navigating = on_navigating.clone();
            view.NavigationStarting(&TypedEventHandler::new(move |_, _| {
                on_navigating.signal::<GlobalRuntime>(());
                Ok(())
            }))?;
        }
        let on_navigated = SendWrapper::new(Rc::new(Callback::new()));
        {
            let on_navigated = on_navigated.clone();
            view.NavigationCompleted(&TypedEventHandler::new(move |_, _| {
                on_navigated.signal::<GlobalRuntime>(());
                Ok(())
            }))?;
        }
        Ok(Self {
            on_navigating,
            on_navigated,
            handle: Widget::new(parent, view.cast()?)?,
            view,
        })
    }

    pub fn is_visible(&self) -> Result<bool>;

    pub fn set_visible(&mut self, v: bool) -> Result<()>;

    pub fn is_enabled(&self) -> Result<bool>;

    pub fn set_enabled(&mut self, v: bool) -> Result<()>;

    pub fn loc(&self) -> Result<Point>;

    pub fn set_loc(&mut self, v: Point) -> Result<()>;

    pub fn size(&self) -> Result<Size>;

    pub fn set_size(&mut self, v: Size) -> Result<()>;

    pub fn source(&self) -> Result<String> {
        Ok(self.view.Source()?.ToString()?.to_string_lossy())
    }

    pub fn set_source(&mut self, s: impl AsRef<str>) -> Result<()> {
        let s = s.as_ref();
        if s.is_empty() {
            return self.set_html("");
        }
        self.view.SetSource(&Uri::CreateUri(&HSTRING::from(s))?)?;
        Ok(())
    }

    pub fn set_html(&mut self, s: impl AsRef<str>) -> Result<()> {
        self.view.NavigateToString(&HSTRING::from(s.as_ref()))?;
        Ok(())
    }

    pub fn can_go_forward(&self) -> Result<bool> {
        self.view.CanGoForward()
    }

    pub fn go_forward(&mut self) -> Result<()> {
        self.view.GoForward()?;
        Ok(())
    }

    pub fn can_go_back(&self) -> Result<bool> {
        self.view.CanGoBack()
    }

    pub fn go_back(&mut self) -> Result<()> {
        self.view.GoBack()?;
        Ok(())
    }

    pub fn reload(&mut self) -> Result<()> {
        self.view.Reload()?;
        Ok(())
    }

    pub fn stop(&mut self) -> Result<()> {
        self.view.CoreWebView2()?.Stop()?;
        Ok(())
    }

    pub async fn wait_navigating(&self) {
        self.on_navigating.wait().await;
    }

    pub async fn wait_navigated(&self) {
        self.on_navigated.wait().await;
    }

    fn cookie_manager(&self) -> Result<CoreWebView2CookieManager> {
        self.view.CoreWebView2()?.CookieManager()
    }

    pub async fn cookies(&self) -> Result<Vec<Cookie<'static>>> {
        let cookies = self.cookie_manager()?.GetCookiesAsync(h!(""))?.await?;
        let mut result = vec![];
        for i in 0..cookies.Size()? {
            let cookie = cookies.GetAt(i)?;
            result.push(webview_cookie_to_cookie(&cookie)?);
        }
        Ok(result)
    }

    pub async fn set_cookie(&mut self, c: &Cookie<'_>) -> Result<()> {
        let manager = self.cookie_manager()?;
        manager.AddOrUpdateCookie(&cookie_to_webview_cookie(c, &manager)?)?;
        Ok(())
    }

    pub async fn delete_cookie(&mut self, c: &Cookie<'_>) -> Result<()> {
        let manager = self.cookie_manager()?;
        manager.DeleteCookie(&cookie_to_webview_cookie(c, &manager)?)?;
        Ok(())
    }

    pub fn run_javascript(
        &mut self,
        s: impl AsRef<str>,
    ) -> Result<impl Future<Output = Result<String>> + 'static> {
        self.view
            .ExecuteScriptAsync(&HSTRING::from(s.as_ref()))
            .map(|fut| fut.into_future().map_ok(|result| result.to_string_lossy()))
    }
}

winio_handle::impl_as_widget!(WebView, handle);

fn cookie_to_webview_cookie(
    c: &Cookie<'_>,
    manager: &CoreWebView2CookieManager,
) -> Result<CoreWebView2Cookie> {
    let cookie = manager.CreateCookie(
        &HSTRING::from(c.name()),
        &HSTRING::from(c.value()),
        &HSTRING::from(c.domain().unwrap_or_default()),
        &HSTRING::from(c.path().unwrap_or_default()),
    )?;
    if let Some(expires) = c.expires() {
        match expires {
            cookie::Expiration::Session => cookie.SetExpires(-1.0)?,
            cookie::Expiration::DateTime(dt) => {
                let timestamp = dt.unix_timestamp() as f64;
                cookie.SetExpires(timestamp)?;
            }
        }
    }
    if let Some(is_secure) = c.secure() {
        cookie.SetIsSecure(is_secure)?;
    }
    if let Some(is_http_only) = c.http_only() {
        cookie.SetIsHttpOnly(is_http_only)?;
    }
    if let Some(same_site) = c.same_site() {
        cookie.SetSameSite(match same_site {
            cookie::SameSite::Lax => CoreWebView2CookieSameSiteKind::Lax,
            cookie::SameSite::Strict => CoreWebView2CookieSameSiteKind::Strict,
            cookie::SameSite::None => CoreWebView2CookieSameSiteKind::None,
        })?;
    }
    Ok(cookie)
}

fn webview_cookie_to_cookie(c: &CoreWebView2Cookie) -> Result<Cookie<'static>> {
    let name = c.Name()?.to_string_lossy();
    let value = c.Value()?.to_string_lossy();
    let domain = c.Domain()?.to_string_lossy();
    let path = c.Path()?.to_string_lossy();
    let expires = c.Expires()?;
    let is_secure = c.IsSecure()?;
    let is_http_only = c.IsHttpOnly()?;
    let same_site = c.SameSite()?;
    let is_session = c.IsSession()?;
    let cookie = Cookie::build((name, value))
        .domain(domain)
        .path(path)
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
            CoreWebView2CookieSameSiteKind::Lax => cookie::SameSite::Lax,
            CoreWebView2CookieSameSiteKind::Strict => cookie::SameSite::Strict,
            CoreWebView2CookieSameSiteKind::None => cookie::SameSite::None,
            _ => return Err(Error::from_hresult(E_INVALIDARG)),
        })
        .build();
    Ok(cookie)
}
