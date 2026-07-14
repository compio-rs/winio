use std::sync::Arc;

use inherit_methods_macro::inherit_methods;
use jni::{
    objects::JObject,
    refs::{LoaderContext, Reference},
};
use jni_min_helper::DynamicProxy;
use winio_callback::SyncCallback;
use winio_handle::{AsContainer, impl_as_widget};
use winio_primitive::{Orient, Point, Size, TickPosition};

use crate::{
    BaseWidget, Result, current_activity,
    java::material::{
        Slider as ASlider, SliderOnChangeListener,
        slider::{HORIZONTAL, TICK_VISIBILITY_AUTO_LIMIT, TICK_VISIBILITY_HIDDEN, VERTICAL},
    },
    vm_exec,
};

#[derive(Debug)]
pub struct Slider {
    inner: BaseWidget<ASlider<'static>>,
    on_change: Arc<SyncCallback>,
    #[allow(dead_code)]
    change_proxy: DynamicProxy,
}

#[inherit_methods(from = "self.inner")]
impl Slider {
    pub fn new(parent: impl AsContainer) -> Result<Self> {
        vm_exec(|env| {
            let act = current_activity(env)?;
            let widget = ASlider::new(env, &act)?;
            let inner = BaseWidget::new_with_env(env, parent.as_container(), widget)?;
            let on_change = Arc::new(SyncCallback::new());
            let change_proxy = DynamicProxy::build(
                env,
                &LoaderContext::FromObject(&act),
                [SliderOnChangeListener::class_name()],
                {
                    let on_change = on_change.clone();
                    move |_env, _this, _args| {
                        on_change.signal(());
                        Ok(JObject::null())
                    }
                },
            )?;
            inner.add_on_change_listener(env, &change_proxy)?;
            Ok(Self {
                inner,
                on_change,
                change_proxy,
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

    pub fn tick_pos(&self) -> Result<TickPosition> {
        let pos = match vm_exec(|env| self.inner.get_tick_visibility_mode(env))? {
            TICK_VISIBILITY_HIDDEN => TickPosition::None,
            _ => TickPosition::Both,
        };
        Ok(pos)
    }

    pub fn set_tick_pos(&mut self, v: TickPosition) -> Result<()> {
        let mode = match v {
            TickPosition::None => TICK_VISIBILITY_HIDDEN,
            _ => TICK_VISIBILITY_AUTO_LIMIT,
        };
        vm_exec(|env| self.inner.set_tick_visibility_mode(env, mode))?;
        Ok(())
    }

    pub fn orient(&self) -> Result<Orient> {
        let orient = match vm_exec(|env| self.inner.is_vertical(env))? {
            true => Orient::Vertical,
            false => Orient::Horizontal,
        };
        Ok(orient)
    }

    pub fn set_orient(&mut self, v: Orient) -> Result<()> {
        let orient = match v {
            Orient::Vertical => VERTICAL,
            Orient::Horizontal => HORIZONTAL,
        };
        vm_exec(|env| self.inner.set_orientation(env, orient))?;
        Ok(())
    }

    pub fn minimum(&self) -> Result<usize> {
        vm_exec(|env| Ok(self.inner.get_value_from(env)? as _))
    }

    pub fn set_minimum(&mut self, v: usize) -> Result<()> {
        vm_exec(|env| self.inner.set_value_from(env, v as _))?;
        Ok(())
    }

    pub fn maximum(&self) -> Result<usize> {
        vm_exec(|env| Ok(self.inner.get_value_to(env)? as _))
    }

    pub fn set_maximum(&mut self, v: usize) -> Result<()> {
        vm_exec(|env| {
            let min = self.inner.get_value_from(env)? as usize;
            let v = if v <= min { min + 1 } else { v };
            self.inner.set_value_to(env, v as _)
        })?;
        Ok(())
    }

    pub fn freq(&self) -> Result<usize> {
        Ok(0)
    }

    pub fn set_freq(&mut self, _v: usize) -> Result<()> {
        Ok(())
    }

    pub fn pos(&self) -> Result<usize> {
        vm_exec(|env| Ok(self.inner.get_value(env)? as _))
    }

    pub fn set_pos(&mut self, v: usize) -> Result<()> {
        vm_exec(|env| {
            let value_from = self.inner.get_value_from(env)?;
            let value_to = self.inner.get_value_to(env)?;
            let v = (v as f32).clamp(value_from, value_to);
            self.inner.set_value(env, v)
        })?;
        Ok(())
    }

    pub async fn wait_change(&self) {
        self.on_change.wait().await;
    }
}

impl_as_widget!(Slider, inner);

#[derive(Debug)]
pub struct ScrollBar {
    handle: Slider,
    page: usize,
}

#[inherit_methods(from = "self.handle")]
impl ScrollBar {
    pub fn new(parent: impl AsContainer) -> Result<Self> {
        let slider = Slider::new(parent)?;
        Ok(Self {
            handle: slider,
            page: 0,
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

    pub fn orient(&self) -> Result<Orient>;

    pub fn set_orient(&mut self, v: Orient) -> Result<()>;

    pub fn minimum(&self) -> Result<usize>;

    pub fn set_minimum(&mut self, v: usize) -> Result<()>;

    pub fn maximum(&self) -> Result<usize> {
        Ok(self.handle.maximum()? + self.page()?)
    }

    pub fn set_maximum(&mut self, v: usize) -> Result<()> {
        self.handle.set_maximum(v.saturating_sub(self.page()?))
    }

    pub fn page(&self) -> Result<usize> {
        Ok(self.page)
    }

    pub fn set_page(&mut self, v: usize) -> Result<()> {
        let max = self.maximum()?;
        self.page = v;
        self.set_maximum(max)
    }

    pub fn pos(&self) -> Result<usize>;

    pub fn set_pos(&mut self, v: usize) -> Result<()>;

    pub async fn wait_change(&self) {
        self.handle.wait_change().await
    }
}

winio_handle::impl_as_widget!(ScrollBar, handle);
