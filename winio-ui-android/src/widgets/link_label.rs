use std::sync::Arc;

use inherit_methods_macro::inherit_methods;
use jni::{
    Env,
    objects::{JObject, JString},
    refs::{Global, LoaderContext, Reference},
};
use jni_min_helper::DynamicProxy;
use winio_callback::SyncCallback;
use winio_handle::AsContainer;
use winio_primitive::{Point, Size};

use crate::{ATextView, BaseWidget, JRunnable, Result, current_activity, vm_exec};

jni::bind_java_type! {
    pub(crate) MovementMethod => android.text.method.MovementMethod,
}

jni::bind_java_type! {
    LinkMovementMethod => android.text.method.LinkMovementMethod,
    type_map {
        MovementMethod => android.text.method.MovementMethod,
    },
    methods {
        static fn get_instance() -> MovementMethod,
    },
    is_instance_of = {
        base = MovementMethod,
    }
}

jni::bind_java_type! {
    ClickableSpan => android.text.style.ClickableSpan,
}

jni::bind_java_type! {
    URLSpan => android.text.style.URLSpan,
    type_map {
        ClickableSpan => android.text.style.ClickableSpan,
    },
    constructors {
        fn new(url: &JString),
    },
    methods {
        fn get_u_r_l() -> JString,
    },
    is_instance_of = {
        base = ClickableSpan,
    }
}

jni::bind_java_type! {
    WinioClickableSpan => rs.compio.winio.ClickableSpan,
    type_map {
        ClickableSpan => android.text.style.ClickableSpan,
        JRunnable => java.lang.Runnable,
    },
    constructors {
        fn new(),
    },
    methods {
        fn set_on_click(listener: &JRunnable),
    },
    is_instance_of = {
        base = ClickableSpan,
    }
}

jni::bind_java_type! {
    SpannableString => android.text.SpannableString,
    constructors {
        fn new(text: &JCharSequence),
    },
    methods {
        fn set_span(what: &JObject, start: i32, end: i32, flags: i32),
        fn to_string() -> JString,
    },
    is_instance_of = {
        char_sequence = JCharSequence,
    }
}

#[derive(Debug)]
pub struct LinkLabel {
    inner: BaseWidget<ATextView<'static>>,
    on_click: Arc<SyncCallback>,
    #[allow(dead_code)]
    click_proxy: DynamicProxy,
    url_span: Global<URLSpan<'static>>,
    click_span: Global<WinioClickableSpan<'static>>,
}

const SPAN_INCLUSIVE_EXCLUSIVE: i32 = 0x11;

#[inherit_methods(from = "self.inner")]
impl LinkLabel {
    pub fn new(parent: impl AsContainer) -> Result<Self> {
        vm_exec(|env| {
            let act = current_activity(env)?;
            let widget = ATextView::new(env, act)?;
            let inner = BaseWidget::new_with_env(env, parent.as_container(), widget)?;

            let method = LinkMovementMethod::get_instance(env)?;
            inner.set_movement_method(env, method)?;

            let on_click = Arc::new(SyncCallback::new());

            let click_span = WinioClickableSpan::new(env)?;
            let click_proxy =
                DynamicProxy::build(env, &LoaderContext::None, [JRunnable::class_name()], {
                    let on_click = on_click.clone();
                    move |_env, _method, _args| {
                        on_click.signal(());
                        Ok(JObject::null())
                    }
                })?;
            click_span.set_on_click(env, &click_proxy)?;
            let click_span = env.new_global_ref(click_span)?;

            let url = JString::new(env, "")?;
            let url_span = URLSpan::new(env, &url)?;
            let url_span = env.new_global_ref(url_span)?;

            Ok(Self {
                inner,
                on_click,
                click_proxy,
                url_span,
                click_span,
            })
        })
    }

    pub fn is_visible(&self) -> Result<bool>;

    pub fn set_visible(&mut self, v: bool) -> Result<()>;

    pub fn is_enabled(&self) -> Result<bool>;

    pub fn set_enabled(&mut self, v: bool) -> Result<()>;

    pub fn preferred_size(&self) -> Result<Size>;

    pub fn loc(&self) -> Result<Point>;

    pub fn set_loc(&mut self, p: Point) -> Result<()>;

    pub fn size(&self) -> Result<Size>;

    pub fn set_size(&mut self, v: Size) -> Result<()>;

    pub fn tooltip(&self) -> Result<String>;

    pub fn set_tooltip(&mut self, s: impl AsRef<str>) -> Result<()>;

    fn update_text_impl(&mut self, env: &mut Env, text: JString, s: &str) -> Result<()> {
        let text = SpannableString::new(env, text.as_char_sequence())?;
        let length = text.as_char_sequence().length(env)?;
        text.set_span(
            env,
            if s.is_empty() {
                self.click_span.as_obj()
            } else {
                self.url_span.as_obj()
            },
            0,
            length,
            SPAN_INCLUSIVE_EXCLUSIVE,
        )?;
        self.inner.set_text(env, text)?;
        Ok(())
    }

    fn text_jstring<'a>(&self, env: &mut Env<'a>) -> Result<JString<'a>> {
        let spannable = self.inner.get_text(env)?;
        let spannable = unsafe { SpannableString::from_raw(env, spannable.into_raw()) };
        let str = spannable.to_string(env)?;
        Ok(str)
    }

    pub fn text(&self) -> Result<String> {
        vm_exec(|env| Ok(self.text_jstring(env)?.try_to_string(env)?))
    }

    pub fn set_text(&mut self, s: impl AsRef<str>) -> Result<()> {
        let uri = self.uri()?;
        vm_exec(|env| {
            let str = JString::new(env, s.as_ref())?;
            self.update_text_impl(env, str, &uri)?;
            Ok(())
        })
    }

    pub fn uri(&self) -> Result<String> {
        vm_exec(|env| Ok(self.url_span.get_u_r_l(env)?.try_to_string(env)?))
    }

    pub fn set_uri(&mut self, s: impl AsRef<str>) -> Result<()> {
        let s = s.as_ref();
        vm_exec(|env| {
            let url = JString::new(env, s)?;
            let url_span = URLSpan::new(env, &url)?;
            self.url_span = env.new_global_ref(url_span)?;
            let str = self.text_jstring(env)?;
            self.update_text_impl(env, str, s)?;
            Ok(())
        })
    }

    pub async fn wait_click(&self) {
        self.on_click.wait().await;
    }
}

winio_handle::impl_as_widget!(LinkLabel, inner);
