use std::sync::Arc;

use inherit_methods_macro::inherit_methods;
use jni::objects::JObject;
use jni_min_helper::DynamicProxy;
use winio_callback::SyncCallback;
use winio_handle::{AsContainer, impl_as_widget};
use winio_primitive::{Point, Size};

use crate::{BaseWidget, Result, vm_exec};

#[derive(Debug)]
pub struct ComboBox {
    inner: BaseWidget,
    on_select: Arc<SyncCallback>,
    #[allow(dead_code)]
    select_proxy: DynamicProxy,
}

// noinspection SpellCheckingInspection
#[inherit_methods(from = "self.inner")]
impl ComboBox {
    const WIDGET_CLASS: &'static str = "android/widget/Spinner";

    pub fn new(parent: impl AsContainer) -> Result<Self> {
        let on_select = Arc::new(SyncCallback::new());
        vm_exec(|env| {
            let select_proxy = DynamicProxy::build(
                env,
                &jni::refs::LoaderContext::None,
                [jni::jni_str!(
                    "android/widget/AdapterView$OnItemSelectedListener"
                )],
                {
                    let on_select = on_select.clone();
                    move |env, method, _args| {
                        if method.get_name(env)?.to_string() == "onItemSelected" {
                            on_select.signal(());
                        }
                        Ok(JObject::null())
                    }
                },
            )?;
            let inner = BaseWidget::new_with_env(env, parent.as_container(), Self::WIDGET_CLASS)?;
            env.call_method(
                inner.as_obj(),
                jni::jni_str!("setOnItemSelectedListener"),
                jni::jni_sig!("(Landroid/widget/AdapterView$OnItemSelectedListener;)V"),
                &[select_proxy.as_ref().into()],
            )?
            .v()?;
            Ok(Self {
                inner,
                on_select,
                select_proxy,
            })
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

    pub fn text(&self) -> Result<String> {
        match self.selection()? {
            Some(i) => self.get(i),
            None => Ok(String::new()),
        }
    }

    pub fn set_text(&mut self, _s: impl AsRef<str>) -> Result<()> {
        Ok(())
    }

    pub fn selection(&self) -> Result<Option<usize>> {
        todo!()
    }

    pub fn set_selection(&mut self, _i: usize) -> Result<()> {
        todo!()
    }

    pub fn is_editable(&self) -> Result<bool> {
        todo!()
    }

    pub fn set_editable(&mut self, _v: bool) -> Result<()> {
        todo!()
    }

    pub fn len(&self) -> Result<usize> {
        todo!()
    }

    pub fn is_empty(&self) -> Result<bool> {
        todo!()
    }

    pub fn clear(&mut self) -> Result<()> {
        todo!()
    }

    pub fn get(&self, _i: usize) -> Result<String> {
        todo!()
    }

    pub fn set(&mut self, _i: usize, _s: impl AsRef<str>) -> Result<()> {
        todo!()
    }

    pub fn insert(&mut self, _i: usize, _s: impl AsRef<str>) -> Result<()> {
        todo!()
    }

    pub fn remove(&mut self, _i: usize) -> Result<()> {
        todo!()
    }

    pub async fn wait_change(&self) {
        std::future::pending().await
    }

    pub async fn wait_select(&self) {
        self.on_select.wait().await
    }
}

impl_as_widget!(ComboBox, inner);
