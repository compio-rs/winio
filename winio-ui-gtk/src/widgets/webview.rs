use std::rc::Rc;

use cookie::{Cookie, Expiration};
use futures_util::TryFutureExt;
use gtk4::glib::object::Cast;
use inherit_methods_macro::inherit_methods;
use webkit6::{prelude::WebViewExt, soup::SameSitePolicy};
use winio_callback::Callback;
use winio_handle::AsContainer;
use winio_primitive::{Point, Size};

use crate::{GlobalRuntime, Result, widgets::Widget};

#[derive(Debug)]
pub struct WebView {
    on_loading: Rc<Callback<()>>,
    on_loaded: Rc<Callback<()>>,
    widget: webkit6::WebView,
    handle: Widget,
}

#[inherit_methods(from = "self.handle")]
impl WebView {
    pub async fn new(parent: impl AsContainer) -> Result<Self> {
        let widget = webkit6::WebView::new();
        let handle = Widget::new(parent, unsafe { widget.clone().unsafe_cast() })?;
        let on_loading = Rc::new(Callback::new());
        widget.connect_resource_load_started({
            let on_loading = on_loading.clone();
            move |_, _, _| {
                on_loading.signal::<GlobalRuntime>(());
            }
        });
        let on_loaded = Rc::new(Callback::new());
        widget.connect_load_changed({
            let on_loaded = on_loaded.clone();
            move |_, _| {
                on_loaded.signal::<GlobalRuntime>(());
            }
        });
        Ok(Self {
            on_loading,
            on_loaded,
            widget,
            handle,
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
        Ok(self.widget.uri().map(|s| s.to_string()).unwrap_or_default())
    }

    pub fn set_source(&mut self, s: impl AsRef<str>) -> Result<()> {
        self.widget.load_uri(s.as_ref());
        Ok(())
    }

    pub fn set_html(&mut self, s: impl AsRef<str>) -> Result<()> {
        self.widget.load_html(s.as_ref(), None);
        Ok(())
    }

    pub fn can_go_forward(&self) -> Result<bool> {
        Ok(self.widget.can_go_forward())
    }

    pub fn go_forward(&mut self) -> Result<()> {
        self.widget.go_forward();
        Ok(())
    }

    pub fn can_go_back(&self) -> Result<bool> {
        Ok(self.widget.can_go_back())
    }

    pub fn go_back(&mut self) -> Result<()> {
        self.widget.go_back();
        Ok(())
    }

    pub fn reload(&mut self) -> Result<()> {
        self.widget.reload();
        Ok(())
    }

    pub fn stop(&mut self) -> Result<()> {
        self.widget.stop_loading();
        Ok(())
    }

    pub async fn wait_navigating(&self) {
        self.on_loading.wait().await
    }

    pub async fn wait_navigated(&self) {
        self.on_loaded.wait().await
    }

    pub async fn cookies(&self) -> Result<Vec<Cookie<'static>>> {
        if let Some(session) = self.widget.network_session()
            && let Some(cookie_manager) = session.cookie_manager()
        {
            let cookies = cookie_manager.all_cookies_future().await?;
            return Ok(cookies
                .into_iter()
                .filter_map(|mut c| cookie_from_soup(&mut c))
                .collect());
        }
        Ok(vec![])
    }

    pub async fn set_cookie(&mut self, c: &Cookie<'_>) -> Result<()> {
        if let Some(session) = self.widget.network_session()
            && let Some(cookie_manager) = session.cookie_manager()
            && let Some(c) = cookie_to_soup(c)
        {
            cookie_manager.add_cookie_future(&c).await?;
        }
        Ok(())
    }

    pub async fn delete_cookie(&mut self, c: &Cookie<'_>) -> Result<()> {
        if let Some(session) = self.widget.network_session()
            && let Some(cookie_manager) = session.cookie_manager()
            && let Some(c) = cookie_to_soup(c)
        {
            cookie_manager.delete_cookie_future(&c).await?;
        }
        Ok(())
    }

    pub fn run_javascript(
        &mut self,
        js: impl AsRef<str>,
    ) -> Result<impl Future<Output = Result<String>> + 'static> {
        let fut = self
            .widget
            .evaluate_javascript_future(js.as_ref(), None, None);
        Ok(fut.map_ok(|s| s.to_string()).map_err(|err| err.into()))
    }
}

winio_handle::impl_as_widget!(WebView, handle);

fn cookie_from_soup(c: &mut webkit6::soup::Cookie) -> Option<Cookie<'static>> {
    let name = c.name().unwrap_or_default();
    let value = c.value().unwrap_or_default();
    let mut builder = Cookie::build((name.to_string(), value.to_string()))
        .secure(c.is_secure())
        .http_only(c.is_http_only())
        .same_site(match c.same_site_policy() {
            SameSitePolicy::Lax => cookie::SameSite::Lax,
            SameSitePolicy::Strict => cookie::SameSite::Strict,
            SameSitePolicy::None => cookie::SameSite::None,
            _ => return None,
        });
    if let Some(s) = c.domain() {
        builder = builder.domain(s.to_string());
    }
    if let Some(s) = c.path() {
        builder = builder.path(s.to_string());
    }
    if let Some(dt) = c.expires() {
        builder = builder.expires(cookie::Expiration::DateTime(
            time::OffsetDateTime::from_unix_timestamp(dt.to_unix()).ok()?,
        ));
    }
    Some(builder.build())
}

fn cookie_to_soup(c: &Cookie<'_>) -> Option<webkit6::soup::Cookie> {
    let mut cookie = webkit6::soup::Cookie::new(
        c.name(),
        c.value(),
        c.domain().unwrap_or_default(),
        c.path().unwrap_or_default(),
        -1,
    );
    if let Some(b) = c.secure() {
        cookie.set_secure(b);
    }
    if let Some(b) = c.http_only() {
        cookie.set_http_only(b);
    }
    if let Some(s) = c.same_site() {
        cookie.set_same_site_policy(match s {
            cookie::SameSite::Lax => SameSitePolicy::Lax,
            cookie::SameSite::Strict => SameSitePolicy::Strict,
            cookie::SameSite::None => SameSitePolicy::None,
        });
    }
    if let Some(Expiration::DateTime(dt)) = c.expires() {
        cookie.set_expires(&gtk4::glib::DateTime::from_unix_utc(dt.unix_timestamp()).ok()?);
    }
    Some(cookie)
}
