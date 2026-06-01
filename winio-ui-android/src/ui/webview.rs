use std::sync::Arc;

use inherit_methods_macro::inherit_methods;
use jni::{objects::JObject, refs::LoaderContext};
use jni_min_helper::DynamicProxy;
use winio_callback::SyncCallback;
use winio_handle::AsContainer;
use winio_primitive::{Point, Size};

use crate::{AView, BaseWidget, Context, Result, current_activity, vm_exec};

jni::bind_java_type! {
    AWebView => android.webkit.WebView,
    type_map {
        AView => android.view.View,
        AWebViewClient => android.webkit.WebViewClient,
        Context => android.content.Context,
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
    },
    is_instance_of = {
        view = AView,
    }
}

jni::bind_java_type! {
    AWebViewClient => android.webkit.WebViewClient,
}

jni::bind_java_type! {
    WinioWebViewClient => rs.compio.winio.WebViewClient,
    type_map {
        AWebViewClient => android.webkit.WebViewClient,
    },
    constructors {
        fn new(),
    },
    is_instance_of = {
        base = AWebViewClient,
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
            let act = current_activity(env)?;
            let widget = AWebView::new(env, act)?;
            let inner = BaseWidget::new_with_env(env, parent.as_container(), widget)?;

            let client = WinioWebViewClient::new(env)?;

            let on_started = Arc::new(SyncCallback::new());
            let started_proxy = DynamicProxy::build(
                env,
                &LoaderContext::None,
                [jni::jni_str!("java/lang/Runnable")],
                {
                    let on_started = on_started.clone();
                    move |_env, _method, _args| {
                        on_started.signal(());
                        Ok(JObject::null())
                    }
                },
            )?;
            env.call_method(
                &client,
                jni::jni_str!("setOnPageStarted"),
                jni::jni_sig!("(Ljava/lang/Runnable;)V"),
                &[started_proxy.as_ref().into()],
            )?;

            let on_finished = Arc::new(SyncCallback::new());
            let finished_proxy = DynamicProxy::build(
                env,
                &LoaderContext::None,
                [jni::jni_str!("java/lang/Runnable")],
                {
                    let on_finished = on_finished.clone();
                    move |_env, _method, _args| {
                        on_finished.signal(());
                        Ok(JObject::null())
                    }
                },
            )?;
            env.call_method(
                &client,
                jni::jni_str!("setOnPageFinished"),
                jni::jni_sig!("(Ljava/lang/Runnable;)V"),
                &[finished_proxy.as_ref().into()],
            )?;

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
        vm_exec(|env| {
            let data = env.new_string(s.as_ref())?;
            let mime = env.new_string("text/html")?;
            let encoding = env.new_string("utf-8")?;
            self.inner.load_data(env, &data, &mime, &encoding)?;
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
}

winio_handle::impl_as_widget!(WebView, inner);
