use std::sync::Arc;

use inherit_methods_macro::inherit_methods;
use jni::{objects::JObject, refs::LoaderContext};
use jni_min_helper::DynamicProxy;
use winio_callback::SyncCallback;
use winio_handle::{AsContainer, impl_as_widget};
use winio_primitive::{Point, Size};

use crate::{BaseWidget, Result, vm_exec};

#[derive(Debug)]
pub struct Button {
    inner: BaseWidget,
    on_click: Arc<SyncCallback>,
}

// noinspection SpellCheckingInspection
#[inherit_methods(from = "self.inner")]
impl Button {
    const WIDGET_CLASS: &'static str = "android/widget/Button";

    pub fn new(parent: impl AsContainer) -> Result<Self> {
        let on_click = Arc::new(SyncCallback::new());
        vm_exec(|env| {
            let proxy = DynamicProxy::build(
                env,
                &LoaderContext::None,
                [jni::jni_str!("android/view/View$OnClickListener")],
                {
                    let on_click = on_click.clone();
                    move |_env, _this, _args| {
                        on_click.signal(());
                        Ok(JObject::null())
                    }
                },
            )?;
            let inner = BaseWidget::new_with_env(env, parent.as_container(), Self::WIDGET_CLASS)?;
            env.call_method(
                inner.as_obj(),
                jni::jni_str!("setOnClickListener"),
                jni::jni_sig!("(Landroid/view/View$OnClickListener;)V"),
                &[proxy.as_ref().into()],
            )?
            .v()?;
            Ok(Self { inner, on_click })
        })
    }

    pub fn is_visible(&self) -> Result<bool>;

    pub fn set_visible(&self, visible: bool) -> Result<()>;

    pub fn is_enabled(&self) -> Result<bool>;

    pub fn set_enabled(&self, enabled: bool) -> Result<()>;

    pub fn loc(&self) -> Result<Point>;

    pub fn set_loc(&self, p: Point) -> Result<()>;

    pub fn size(&self) -> Result<Size>;

    pub fn set_size(&self, v: Size) -> Result<()>;

    pub fn preferred_size(&self) -> Result<Size>;

    pub fn text(&self) -> Result<String>;

    pub fn set_text(&self, text: impl AsRef<str>) -> Result<()>;

    pub async fn wait_click(&self) {
        self.on_click.wait().await;
    }
}

impl_as_widget!(Button, inner);
