use std::sync::Arc;

use inherit_methods_macro::inherit_methods;
use jni::{
    Env,
    objects::{JObject, JString},
    refs::LoaderContext,
};
use jni_min_helper::DynamicProxy;
use winio_callback::SyncCallback;
use winio_handle::AsContainer;
use winio_primitive::{Point, Size};

use crate::{ATextView, BaseWidget, GlobalRef, JObjectExt, Result, current_activity, vm_exec};

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

#[derive(Debug)]
pub struct LinkLabel {
    inner: BaseWidget<ATextView<'static>>,
    on_click: Arc<SyncCallback>,
    #[allow(dead_code)]
    click_proxy: DynamicProxy,
    url_span: GlobalRef,
    click_span: GlobalRef,
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

            let click_span_class_name = JString::from_str(env, "rs/compio/winio/ClickableSpan")?;
            let click_span_class =
                crate::winio_class_loader()?.load_class(env, click_span_class_name)?;
            let click_span = env.new_object(click_span_class, jni::jni_sig!("()V"), &[])?;
            let click_proxy = DynamicProxy::build(
                env,
                &LoaderContext::None,
                [jni::jni_str!("java/lang/Runnable")],
                {
                    let on_click = on_click.clone();
                    move |_env, _method, _args| {
                        on_click.signal(());
                        Ok(JObject::null())
                    }
                },
            )?;
            env.call_method(
                &click_span,
                jni::jni_str!("setOnClick"),
                jni::jni_sig!("(Ljava/lang/Runnable;)V"),
                &[click_proxy.as_ref().into()],
            )?
            .v()?;
            let click_span = env.new_global_ref(click_span)?;

            let url = JString::new(env, "")?;
            let url_span = env.new_object(
                jni::jni_str!("android/text/style/URLSpan"),
                jni::jni_sig!("(Ljava/lang/String;)V"),
                &[(&url).into()],
            )?;
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

    fn update_text_impl(&mut self, env: &mut Env, text: JObject, s: &str) -> Result<()> {
        let text = env.new_object(
            jni::jni_str!("android/text/SpannableString"),
            jni::jni_sig!("(Ljava/lang/CharSequence;)V"),
            &[(&text).into()],
        )?;
        let text = unsafe { JString::from_raw(env, text.into_raw()) };
        let length = text.as_char_sequence().length(env)?;
        compio_log::info!("update_text_impl: text={:?}, s={:?}", length, s);
        env.call_method(
            &text,
            jni::jni_str!("setSpan"),
            jni::jni_sig!("(Ljava/lang/Object;III)V"),
            &[
                (if s.is_empty() {
                    self.click_span.as_obj()
                } else {
                    self.url_span.as_obj()
                })
                .into(),
                0i32.into(),
                length.into(),
                SPAN_INCLUSIVE_EXCLUSIVE.into(),
            ],
        )?
        .v()?;
        env.call_method(
            self.inner.as_obj(),
            jni::jni_str!("setText"),
            jni::jni_sig!("(Ljava/lang/CharSequence;)V"),
            &[(&text).into()],
        )?
        .v()?;
        Ok(())
    }

    fn text_jstring<'a>(&self, env: &mut Env<'a>) -> Result<JObject<'a>> {
        let spannable = env
            .call_method(
                self.inner.as_obj(),
                jni::jni_str!("getText"),
                jni::jni_sig!("()Ljava/lang/CharSequence;"),
                &[],
            )?
            .l()?;
        let str = env
            .call_method(
                &spannable,
                jni::jni_str!("toString"),
                jni::jni_sig!("()Ljava/lang/String;"),
                &[],
            )?
            .l()?;
        Ok(str)
    }

    pub fn text(&self) -> Result<String> {
        vm_exec(|env| self.text_jstring(env)?.to(env))
    }

    pub fn set_text(&mut self, s: impl AsRef<str>) -> Result<()> {
        let uri = self.uri()?;
        vm_exec(|env| {
            let str = JString::new(env, s.as_ref())?;
            self.update_text_impl(env, str.into(), &uri)?;
            Ok(())
        })
    }

    pub fn uri(&self) -> Result<String> {
        vm_exec(|env| {
            let url = env
                .call_method(
                    &self.url_span,
                    jni::jni_str!("getURL"),
                    jni::jni_sig!("()Ljava/lang/String;"),
                    &[],
                )?
                .l()?
                .to(env)?;
            Ok(url)
        })
    }

    pub fn set_uri(&mut self, s: impl AsRef<str>) -> Result<()> {
        let s = s.as_ref();
        vm_exec(|env| {
            let url = JString::new(env, s)?;
            let url_span = env.new_object(
                jni::jni_str!("android/text/style/URLSpan"),
                jni::jni_sig!("(Ljava/lang/String;)V"),
                &[(&url).into()],
            )?;
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
