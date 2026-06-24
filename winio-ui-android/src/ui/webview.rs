use std::sync::{Arc, Mutex};

use cookie::Cookie;
use inherit_methods_macro::inherit_methods;
use jni::{
    objects::{JObject, JString},
    refs::{LoaderContext, Reference},
};
use jni_min_helper::DynamicProxy;
use winio_callback::SyncCallback;
use winio_handle::AsContainer;
use winio_primitive::{Point, Size};

use crate::{
    AView, BaseWidget, Context, JRunnable, Result, current_activity, impl_listener, vm_exec,
};

jni::bind_java_type! {
    AWebView => android.webkit.WebView,
    type_map {
        AView => android.view.View,
        AWebViewClient => android.webkit.WebViewClient,
        AWebSettings => android.webkit.WebSettings,
        AWebChromeClient => android.webkit.WebChromeClient,
        Context => android.content.Context,
        ValueCallback => android.webkit.ValueCallback,
    },
    constructors {
        fn new(context: &Context),
    },
    methods {
        fn get_url() -> JString,
        fn load_url(url: &JString),
        fn load_data(data: &JString, mime: &JString, encoding: &JString),
        fn can_go_forward() -> jboolean,
        fn go_forward(),
        fn can_go_back() -> jboolean,
        fn go_back(),
        fn reload(),
        fn stop_loading(),
        fn set_web_view_client(client: &AWebViewClient),
        fn get_settings() -> AWebSettings,
        fn set_web_chrome_client(client: &AWebChromeClient),
        fn evaluate_javascript(script: &JString, callback: &ValueCallback),
    },
    is_instance_of = {
        view = AView,
    }
}

jni::bind_java_type! {
    AWebSettings => android.webkit.WebSettings,
    methods {
        fn set_java_script_enabled(enabled: bool),
    }
}

jni::bind_java_type! {
    ValueCallback => android.webkit.ValueCallback,
}

impl_listener!(ValueCallback);

jni::bind_java_type! {
    AWebChromeClient => android.webkit.WebChromeClient,
    constructors {
        fn new(),
    },
}

jni::bind_java_type! {
    AWebViewClient => android.webkit.WebViewClient,
}

jni::bind_java_type! {
    WinioWebViewClient => rs.compio.winio.WebViewClient,
    type_map {
        AWebViewClient => android.webkit.WebViewClient,
        JRunnable => java.lang.Runnable,
    },
    constructors {
        fn new(),
    },
    methods {
        fn set_on_page_started(listener: &JRunnable),
        fn set_on_page_finished(listener: &JRunnable),
    },
    is_instance_of = {
        base = AWebViewClient,
    }
}

jni::bind_java_type! {
    CookieManager => android.webkit.CookieManager,
    type_map {
        ValueCallback => android.webkit.ValueCallback,
    },
    methods {
        static fn get_instance() -> CookieManager,

        fn set_accept_cookie(accept: bool),
        fn set_cookie(url: &JString, value: &JString, callback: &ValueCallback),
        fn get_cookie(url: &JString) -> JString,
    }
}

#[derive(Debug)]
pub struct WebView {
    inner: BaseWidget<AWebView<'static>>,
    on_started: Arc<SyncCallback>,
    #[allow(dead_code)]
    started_proxy: DynamicProxy,
    on_finished: Arc<SyncCallback>,
    #[allow(dead_code)]
    finished_proxy: DynamicProxy,
}

#[inherit_methods(from = "self.inner")]
impl WebView {
    pub async fn new(parent: impl AsContainer) -> Result<Self> {
        vm_exec(|env| {
            CookieManager::get_instance(env)?.set_accept_cookie(env, true)?;

            let act = current_activity(env)?;
            let widget = AWebView::new(env, act)?;
            widget
                .get_settings(env)?
                .set_java_script_enabled(env, true)?;
            let chrome_client = AWebChromeClient::new(env)?;
            widget.set_web_chrome_client(env, &chrome_client)?;
            let inner = BaseWidget::new_with_env(env, parent.as_container(), widget)?;

            let client = WinioWebViewClient::new(env)?;

            let on_started = Arc::new(SyncCallback::new());
            let started_proxy =
                DynamicProxy::build(env, &LoaderContext::None, [JRunnable::class_name()], {
                    let on_started = on_started.clone();
                    move |_env, _method, _args| {
                        on_started.signal(());
                        Ok(JObject::null())
                    }
                })?;
            client.set_on_page_started(env, &started_proxy)?;

            let on_finished = Arc::new(SyncCallback::new());
            let finished_proxy =
                DynamicProxy::build(env, &LoaderContext::None, [JRunnable::class_name()], {
                    let on_finished = on_finished.clone();
                    move |_env, _method, _args| {
                        on_finished.signal(());
                        Ok(JObject::null())
                    }
                })?;
            client.set_on_page_finished(env, &finished_proxy)?;

            inner.set_web_view_client(env, client)?;

            Ok(Self {
                inner,
                on_started,
                started_proxy,
                on_finished,
                finished_proxy,
            })
        })
    }

    pub fn is_visible(&self) -> Result<bool>;

    pub fn set_visible(&mut self, v: bool) -> Result<()>;

    pub fn is_enabled(&self) -> Result<bool>;

    pub fn set_enabled(&mut self, v: bool) -> Result<()>;

    pub fn loc(&self) -> Result<Point>;

    pub fn set_loc(&mut self, p: Point) -> Result<()>;

    pub fn size(&self) -> Result<Size>;

    pub fn set_size(&mut self, v: Size) -> Result<()>;

    pub fn source(&self) -> Result<String> {
        vm_exec(|env| Ok(self.inner.get_url(env)?.try_to_string(env)?))
    }

    pub fn set_source(&mut self, s: impl AsRef<str>) -> Result<()> {
        vm_exec(|env| {
            let url = env.new_string(s.as_ref())?;
            self.inner.load_url(env, &url)?;
            Ok(())
        })
    }

    pub fn set_html(&mut self, s: impl AsRef<str>) -> Result<()> {
        use base64::prelude::*;

        let s = BASE64_STANDARD.encode(s.as_ref());
        vm_exec(|env| {
            let data = env.new_string(s)?;
            let encoding = env.new_string("base64")?;
            self.inner
                .load_data(env, &data, JString::null(), &encoding)?;
            Ok(())
        })
    }

    pub fn can_go_forward(&self) -> Result<bool> {
        vm_exec(|env| Ok(self.inner.can_go_forward(env)?))
    }

    pub fn go_forward(&mut self) -> Result<()> {
        vm_exec(|env| {
            self.inner.go_forward(env)?;
            Ok(())
        })
    }

    pub fn can_go_back(&self) -> Result<bool> {
        vm_exec(|env| Ok(self.inner.can_go_back(env)?))
    }

    pub fn go_back(&mut self) -> Result<()> {
        vm_exec(|env| {
            self.inner.go_back(env)?;
            Ok(())
        })
    }

    pub fn reload(&mut self) -> Result<()> {
        vm_exec(|env| {
            self.inner.reload(env)?;
            Ok(())
        })
    }

    pub fn stop(&mut self) -> Result<()> {
        vm_exec(|env| {
            self.inner.stop_loading(env)?;
            Ok(())
        })
    }

    pub async fn wait_navigating(&self) {
        self.on_started.wait().await;
    }

    pub async fn wait_navigated(&self) {
        self.on_finished.wait().await;
    }

    pub async fn cookies(&self) -> Result<Vec<Cookie<'static>>> {
        let source = self.source()?;
        if source.is_empty() {
            return Ok(vec![]);
        }
        vm_exec(|env| {
            let url = env.new_string(&source)?;
            let cookies = CookieManager::get_instance(env)?.get_cookie(env, &url)?;
            let cookies = cookies.try_to_string(env)?;
            if let Ok(cookie) = Cookie::parse(cookies) {
                Ok(vec![cookie])
            } else {
                Ok(vec![])
            }
        })
    }

    async fn set_cookie_impl(&mut self, cookie: &str) -> Result<()> {
        let source = self.source()?;
        if source.is_empty() {
            return Ok(());
        }
        let (rx, _proxy) = vm_exec(|env| {
            let url = env.new_string(&source)?;
            let value = env.new_string(cookie)?;
            let (tx, rx) = oneshot::channel();
            let tx = Arc::new(Mutex::new(Some(tx)));
            let proxy =
                DynamicProxy::build(env, &LoaderContext::None, [ValueCallback::class_name()], {
                    move |_env, _method, _args| {
                        if let Some(tx) = tx.lock().unwrap().take() {
                            tx.send(()).ok();
                        }
                        Ok(JObject::null())
                    }
                })?;
            CookieManager::get_instance(env)?.set_cookie(env, &url, &value, &proxy)?;
            Result::Ok((rx, proxy))
        })?;
        rx.await.map_err(std::io::Error::other)?;
        Ok(())
    }

    pub async fn set_cookie(&mut self, c: &Cookie<'_>) -> Result<()> {
        self.set_cookie_impl(&c.to_string()).await
    }

    pub async fn delete_cookie(&mut self, c: &Cookie<'_>) -> Result<()> {
        let mut cookie = c.clone();
        cookie.set_max_age(time::Duration::seconds(0));
        self.set_cookie_impl(&cookie.to_string()).await
    }

    pub async fn run_javascript(&mut self, script: impl AsRef<str>) -> Result<String> {
        let (tx, rx) = oneshot::channel();
        let _proxy = vm_exec(|env| {
            let script = env.new_string(script.as_ref())?;
            let tx = Arc::new(Mutex::new(Some(tx)));
            let callback =
                DynamicProxy::build(env, &LoaderContext::None, [ValueCallback::class_name()], {
                    move |env, method, args| {
                        let name = method.get_name(env)?.try_to_string(env)?;
                        if name == "onReceiveValue" {
                            let value = args.get_element(env, 0)?;
                            let value = env.cast_local::<JString>(value)?;
                            let result = if value.is_null() {
                                None
                            } else {
                                Some(value.try_to_string(env)?)
                            };
                            if let Some(tx) = tx.lock().unwrap().take() {
                                tx.send(result).ok();
                            }
                        }
                        Ok(JObject::null())
                    }
                })?;
            self.inner.evaluate_javascript(env, &script, &callback)?;
            Result::Ok(callback)
        })?;
        Ok(rx.await.map_err(std::io::Error::other)?.unwrap_or_default())
    }
}

winio_handle::impl_as_widget!(WebView, inner);
