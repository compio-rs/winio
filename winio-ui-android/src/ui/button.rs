use std::sync::Arc;

use inherit_methods_macro::inherit_methods;
use jni::{objects::JObject, refs::LoaderContext};
use jni_min_helper::DynamicProxy;
use winio_callback::SyncCallback;
use winio_handle::{AsContainer, impl_as_widget};
use winio_primitive::{Point, Size};

use crate::{BaseWidget, Result, vm_exec};

#[derive(Debug)]
struct ButtonImpl {
    inner: BaseWidget,
    on_click: Arc<SyncCallback>,
}

#[inherit_methods(from = "self.inner")]
impl ButtonImpl {
    pub fn new(parent: impl AsContainer, class: &str) -> Result<Self> {
        let on_click = Arc::new(SyncCallback::new());
        vm_exec(|env| {
            let proxy = DynamicProxy::build(
                env,
                &LoaderContext::None,
                [jni::jni_str!("android/view/View$OnClickListener")],
                {
                    let on_click = on_click.clone();
                    move |_env, _method, _args| {
                        on_click.signal(());
                        Ok(JObject::null())
                    }
                },
            )?;
            let inner = BaseWidget::new_with_env(env, parent.as_container(), class)?;
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

    pub fn set_visible(&mut self, visible: bool) -> Result<()>;

    pub fn is_enabled(&self) -> Result<bool>;

    pub fn set_enabled(&mut self, enabled: bool) -> Result<()>;

    pub fn loc(&self) -> Result<Point>;

    pub fn set_loc(&mut self, p: Point) -> Result<()>;

    pub fn size(&self) -> Result<Size>;

    pub fn set_size(&mut self, v: Size) -> Result<()>;

    pub fn preferred_size(&self) -> Result<Size>;

    pub fn tooltip(&self) -> Result<String>;

    pub fn set_tooltip(&mut self, s: impl AsRef<str>) -> Result<()>;

    pub fn text(&self) -> Result<String>;

    pub fn set_text(&mut self, text: impl AsRef<str>) -> Result<()>;

    pub fn is_checked(&self) -> Result<bool> {
        vm_exec(|env| {
            Ok(env
                .call_method(
                    self.inner.as_obj(),
                    jni::jni_str!("isChecked"),
                    jni::jni_sig!("()Z"),
                    &[],
                )?
                .z()?)
        })
    }

    pub fn set_checked(&mut self, checked: bool) -> Result<()> {
        vm_exec(|env| {
            env.call_method(
                self.inner.as_obj(),
                jni::jni_str!("setChecked"),
                jni::jni_sig!("(Z)V"),
                &[checked.into()],
            )?
            .v()?;
            Ok(())
        })
    }

    pub async fn wait_click(&self) {
        self.on_click.wait().await;
    }
}

impl_as_widget!(ButtonImpl, inner);

#[derive(Debug)]
pub struct Button {
    inner: ButtonImpl,
}

#[inherit_methods(from = "self.inner")]
impl Button {
    const WIDGET_CLASS: &'static str = "android/widget/Button";

    pub fn new(parent: impl AsContainer) -> Result<Self> {
        Ok(Self {
            inner: ButtonImpl::new(parent, Self::WIDGET_CLASS)?,
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

    pub fn text(&self) -> Result<String>;

    pub fn set_text(&mut self, text: impl AsRef<str>) -> Result<()>;

    pub async fn wait_click(&self) {
        self.inner.wait_click().await;
    }
}

impl_as_widget!(Button, inner);

#[derive(Debug)]
pub struct CheckBox {
    inner: ButtonImpl,
}

#[inherit_methods(from = "self.inner")]
impl CheckBox {
    const WIDGET_CLASS: &'static str = "android/widget/CheckBox";

    pub fn new(parent: impl AsContainer) -> Result<Self> {
        Ok(Self {
            inner: ButtonImpl::new(parent, Self::WIDGET_CLASS)?,
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

    pub fn text(&self) -> Result<String>;

    pub fn set_text(&mut self, text: impl AsRef<str>) -> Result<()>;

    pub fn is_checked(&self) -> Result<bool>;

    pub fn set_checked(&mut self, checked: bool) -> Result<()>;

    pub async fn wait_click(&self) {
        self.inner.wait_click().await;
    }
}

impl_as_widget!(CheckBox, inner);

#[derive(Debug)]
pub struct RadioButton {
    inner: ButtonImpl,
}

// noinspection SpellCheckingInspection
#[inherit_methods(from = "self.inner")]
impl RadioButton {
    const WIDGET_CLASS: &'static str = "android/widget/RadioButton";

    pub fn new(parent: impl AsContainer) -> Result<Self> {
        Ok(Self {
            inner: ButtonImpl::new(parent.as_container(), Self::WIDGET_CLASS)?,
        })
    }

    pub fn is_visible(&self) -> Result<bool>;

    pub fn set_visible(&mut self, visible: bool) -> Result<()>;

    pub fn loc(&self) -> Result<Point>;

    pub fn set_loc(&mut self, p: Point) -> Result<()>;

    pub fn size(&self) -> Result<Size>;

    pub fn set_size(&mut self, v: Size) -> Result<()>;

    pub fn preferred_size(&self) -> Result<Size>;

    pub fn is_enabled(&self) -> Result<bool>;

    pub fn set_enabled(&mut self, enabled: bool) -> Result<()>;

    pub fn text(&self) -> Result<String>;

    pub fn set_text(&mut self, s: impl AsRef<str>) -> Result<()>;

    pub fn is_checked(&self) -> Result<bool>;

    pub fn set_checked(&mut self, _v: bool) -> Result<()>;

    pub async fn wait_click(&self) {
        self.inner.wait_click().await
    }
}

impl_as_widget!(RadioButton, inner);
