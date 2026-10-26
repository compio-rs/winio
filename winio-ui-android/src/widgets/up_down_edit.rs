use std::sync::Arc;

use inherit_methods_macro::inherit_methods;
use jni::{
    objects::JObject,
    refs::{LoaderContext, Reference},
};
use jni_min_helper::DynamicProxy;
use winio_callback::SyncCallback;
use winio_handle::{AsContainer, impl_as_widget};
use winio_primitive::{Point, Size};

use crate::{
    BaseWidget, Result, current_activity,
    java::android::widget::{NumberPicker as ANumberPicker, NumberPickerOnValueChangeListener},
    runtime::AndroidRuntime,
    vm_exec,
};

#[derive(Debug)]
pub struct UpDownEdit {
    inner: BaseWidget<ANumberPicker<'static>>,
    on_change: Arc<SyncCallback>,
    #[allow(dead_code)]
    change_proxy: DynamicProxy,
    step: usize,
}

#[inherit_methods(from = "self.inner")]
impl UpDownEdit {
    pub fn new(parent: impl AsContainer) -> Result<Self> {
        vm_exec(|env| {
            let act = current_activity(env)?;
            let widget = ANumberPicker::new(env, &act)?;
            widget.set_wrap_selector_wheel(env, false)?;
            let on_change = Arc::new(SyncCallback::new());
            let change_proxy = DynamicProxy::build(
                env,
                &LoaderContext::FromObject(&act),
                [NumberPickerOnValueChangeListener::class_name()],
                {
                    let on_change = on_change.clone();
                    move |_env, _this, _args| {
                        on_change.signal::<AndroidRuntime>(());
                        Ok(JObject::null())
                    }
                },
            )?;
            widget.set_on_value_changed_listener(env, &change_proxy)?;
            let inner = BaseWidget::new_with_env(env, parent.as_container(), widget)?;
            Ok(Self {
                inner,
                on_change,
                change_proxy,
                step: 1,
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

    pub fn minimum(&self) -> Result<usize> {
        vm_exec(|env| Ok(self.inner.get_min_value(env)? as _))
    }

    pub fn set_minimum(&mut self, v: usize) -> Result<()> {
        vm_exec(|env| {
            self.inner.set_min_value(env, v as _)?;
            Ok(())
        })
    }

    pub fn maximum(&self) -> Result<usize> {
        vm_exec(|env| Ok(self.inner.get_max_value(env)? as _))
    }

    pub fn set_maximum(&mut self, v: usize) -> Result<()> {
        vm_exec(|env| {
            self.inner.set_max_value(env, v as _)?;
            Ok(())
        })
    }

    pub fn step(&self) -> Result<usize> {
        Ok(self.step)
    }

    pub fn set_step(&mut self, v: usize) -> Result<()> {
        self.step = v;
        Ok(())
    }

    pub fn value(&self) -> Result<usize> {
        vm_exec(|env| Ok(self.inner.get_value(env)? as _))
    }

    pub fn set_value(&mut self, v: usize) -> Result<()> {
        vm_exec(|env| {
            self.inner.set_value(env, v as _)?;
            Ok(())
        })
    }

    pub async fn wait_change(&self) {
        self.on_change.wait().await
    }
}

impl_as_widget!(UpDownEdit, inner);
