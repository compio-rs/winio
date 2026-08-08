use std::sync::Arc;

use inherit_methods_macro::inherit_methods;
use jni::{
    Env,
    objects::{JObject, JString},
    refs::{Global, LoaderContext, Reference},
    sys::jfloat,
};
use jni_min_helper::DynamicProxy;
use winio_callback::SyncCallback;
use winio_handle::AsContainer;
use winio_primitive::{Font, Point, Size};

use crate::{
    BaseWidget, Result, current_activity,
    java::{
        android::{
            graphics::{Typeface, typeface},
            text::{
                SpannableString, method::LinkMovementMethod, spanned::SPAN_INCLUSIVE_EXCLUSIVE,
                style::URLSpan,
            },
            widget::TextView as ATextView,
        },
        custom::WinioClickableSpan,
        lang::JRunnable,
    },
    vm_exec,
};

#[derive(Debug)]
pub struct LinkLabel {
    inner: BaseWidget<ATextView<'static>>,
    on_click: Arc<SyncCallback>,
    #[allow(dead_code)]
    click_proxy: DynamicProxy,
    url_span: Global<URLSpan<'static>>,
    click_span: Global<WinioClickableSpan<'static>>,
}

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

    pub fn font(&self) -> Result<Font> {
        vm_exec(|env| {
            let paint = self.inner.get_paint(env)?;
            let px = paint.get_text_size(env)?;
            let metrics = self
                .inner
                .as_view()
                .get_resources(env)?
                .get_display_metrics(env)?;
            let typeface = paint.get_typeface(env)?;
            let family = self.inner.get_font_family(env)?;
            let family = if family.is_null() {
                String::new()
            } else {
                let family = unsafe { JString::from_raw(env, family.into_raw()) };
                family.try_to_string(env)?
            };
            Ok(Font {
                family,
                size: px as f64 / metrics.scaled_density(env)? as f64,
                bold: typeface.is_bold(env)?,
                italic: typeface.is_italic(env)?,
            })
        })
    }

    pub fn set_font(&mut self, font: Font) -> Result<()> {
        vm_exec(|env| {
            let mut style = typeface::NORMAL;
            if font.bold {
                style |= typeface::BOLD;
            }
            if font.italic {
                style |= typeface::ITALIC;
            }
            let family = env.new_string(&font.family)?;
            self.inner.set_text_size(env, font.size as jfloat)?;
            let default = Typeface::DEFAULT(env)?;
            self.inner.set_font_family(env, &family)?;
            self.inner.set_typeface_style(env, &default, style)?;
            Ok(())
        })
    }
}

winio_handle::impl_as_widget!(LinkLabel, inner);
