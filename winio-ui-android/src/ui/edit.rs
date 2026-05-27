use std::sync::Arc;

use inherit_methods_macro::inherit_methods;
use jni::{jni_sig, jni_str, objects::JObject};
use winio_callback::SyncCallback;
use winio_handle::{AsContainer, impl_as_widget};
use winio_primitive::{HAlign, Point, Size};

use crate::{BaseWidget, JObjectExt, Result, vm_exec};

#[derive(Debug)]
pub struct Edit {
    inner: BaseWidget,
    on_change: Arc<SyncCallback>,
}

mod input_type {
    pub const TYPE_CLASS_TEXT: i32 = 0x1;
    pub const TYPE_TEXT_VARIATION_NORMAL: i32 = 0x0;
    pub const TYPE_TEXT_VARIATION_PASSWORD: i32 = 0x80;
}

// noinspection SpellCheckingInspection
#[inherit_methods(from = "self.inner")]
impl Edit {
    const WIDGET_CLASS: &'static str = "android/widget/EditText";

    pub fn new(parent: impl AsContainer) -> Result<Self> {
        let on_change = Arc::new(SyncCallback::new());
        vm_exec(|env| {
            let proxy = jni_min_helper::DynamicProxy::build(
                env,
                &jni::refs::LoaderContext::None,
                [jni_str!("android/text/TextWatcher")],
                {
                    let on_change = on_change.clone();
                    move |env, method, _args| {
                        if method.get_name(env)?.to_string() == "onTextChanged" {
                            on_change.signal(());
                        }
                        Ok(JObject::null())
                    }
                },
            )?;
            let inner = BaseWidget::new_with_env(env, parent.as_container(), Self::WIDGET_CLASS)?;
            env.call_method(
                inner.as_obj(),
                jni::jni_str!("addTextChangedListener"),
                jni::jni_sig!("(Landroid/text/TextWatcher;)V"),
                &[proxy.as_ref().into()],
            )?
            .v()?;
            Ok(Self { inner, on_change })
        })
    }

    pub fn is_visible(&self) -> Result<bool>;

    pub fn set_visible(&mut self, visible: bool) -> Result<()>;

    pub fn is_enabled(&self) -> Result<bool>;

    pub fn set_enabled(&mut self, enabled: bool) -> Result<()>;

    pub fn loc(&self) -> Result<Point>;

    pub fn set_loc(&mut self, p: Point) -> Result<()>;

    pub fn size(&self) -> Result<Size>;

    pub fn set_size(&mut self, v: Size) -> Result<()>;

    pub fn preferred_size(&self) -> Result<Size>;

    pub fn text(&self) -> Result<String> {
        vm_exec(move |env| {
            env.call_method(
                self.inner.as_obj(),
                jni_str!("getTextString"),
                jni_sig!("()Ljava/lang/CharSequence;"),
                &[],
            )?
            .l()?
            .to(env)
        })
    }

    pub fn set_text(&mut self, text: impl AsRef<str>) -> Result<()>;

    pub fn halign(&self) -> Result<HAlign>;

    pub fn set_halign(&mut self, align: HAlign) -> Result<()>;

    pub fn is_password(&self) -> Result<bool> {
        vm_exec(move |env| {
            let ty = env
                .call_method(
                    self.inner.as_obj(),
                    jni_str!("getInputType"),
                    jni_sig!("()I"),
                    &[],
                )?
                .i()?;
            Ok((ty & input_type::TYPE_TEXT_VARIATION_PASSWORD) != 0)
        })
    }

    pub fn set_password(&mut self, password: bool) -> Result<()> {
        vm_exec(move |env| {
            let ty = if password {
                input_type::TYPE_CLASS_TEXT | input_type::TYPE_TEXT_VARIATION_PASSWORD
            } else {
                input_type::TYPE_CLASS_TEXT | input_type::TYPE_TEXT_VARIATION_NORMAL
            };
            env.call_method(
                self.inner.as_obj(),
                jni_str!("setInputType"),
                jni_sig!("(I)V"),
                &[ty.into()],
            )?
            .v()?;
            Ok(())
        })
    }

    pub async fn wait_change(&self) {
        self.on_change.wait().await;
    }
}

impl_as_widget!(Edit, inner);
