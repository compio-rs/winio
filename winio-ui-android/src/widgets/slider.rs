use std::sync::Arc;

use inherit_methods_macro::inherit_methods;
use jni::{
    objects::JObject,
    refs::{LoaderContext, Reference},
};
use jni_min_helper::DynamicProxy;
use winio_callback::SyncCallback;
use winio_handle::{AsContainer, impl_as_widget};
use winio_primitive::{Orient, Point, Size, TickPosition, fit_page};

use crate::{
    BaseWidget, Result, current_activity,
    java::material::{
        Slider as ASlider, SliderOnChangeListener,
        slider::{HORIZONTAL, TICK_VISIBILITY_AUTO_LIMIT, TICK_VISIBILITY_HIDDEN, VERTICAL},
    },
    runtime::AndroidRuntime,
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
                        on_change.signal::<AndroidRuntime>(());
                        Ok(JObject::null())
                    }
                },
            )?;
            inner.add_on_change_listener(env, &change_proxy)?;
            // Initial range: 0..=10, value 0.
            inner.set_value_to(env, 10.0)?;
            inner.set_value_from(env, 0.0)?;
            inner.set_value(env, 0.0)?;
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
        let max = self.maximum()?.max(v.saturating_add(1));
        self.set_range(v, max)?;
        self.set_pos(self.pos()?)?;
        Ok(())
    }

    pub fn maximum(&self) -> Result<usize> {
        vm_exec(|env| Ok(self.inner.get_value_to(env)? as _))
    }

    pub fn set_maximum(&mut self, v: usize) -> Result<()> {
        let old_min = self.minimum()?;
        let min = if v < old_min.saturating_add(1) {
            v.saturating_sub(1)
        } else {
            old_min
        };
        let max = v.max(min.saturating_add(1));
        self.set_range(min, max)?;
        self.set_pos(self.pos()?)?;
        Ok(())
    }

    pub fn freq(&self) -> Result<usize> {
        Ok(1)
    }

    pub fn set_freq(&mut self, _v: usize) -> Result<()> {
        compio_log::error!("Tick frequency is not supported on Android");
        Ok(())
    }

    pub fn pos(&self) -> Result<usize> {
        vm_exec(|env| Ok(self.inner.get_value(env)? as _))
    }

    pub fn set_pos(&mut self, v: usize) -> Result<()> {
        let min = self.minimum()?;
        let max = self.maximum()?;
        vm_exec(|env| self.inner.set_value(env, v.clamp(min, max) as _))?;
        Ok(())
    }

    /// Set both ends of the range, keeping it valid at every step.
    fn set_range(&mut self, min: usize, max: usize) -> Result<()> {
        vm_exec(|env| {
            let old_max = self.inner.get_value_to(env)? as usize;
            if min > old_max {
                self.inner.set_value_to(env, max as _)?;
                self.inner.set_value_from(env, min as _)?;
            } else {
                self.inner.set_value_from(env, min as _)?;
                self.inner.set_value_to(env, max as _)?;
            }
            Ok(())
        })
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
    /// The public content extent. The inner Material slider only stores the
    /// scrollable part (`max - page`), and when the page fills the range it
    /// needs one hidden extra value (`min + 1`) because Material requires
    /// `valueFrom < valueTo`. The public maximum therefore cannot be derived
    /// from the inner slider.
    max: usize,
}

#[inherit_methods(from = "self.handle")]
impl ScrollBar {
    pub fn new(parent: impl AsContainer) -> Result<Self> {
        let mut slider = Slider::new(parent)?;
        // Initial range: min 0, content extent 10, page 1, pos 0.
        // The inner slider only needs the scrollable part.
        slider.set_maximum(9)?;
        Ok(Self {
            handle: slider,
            page: 1,
            max: 10,
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

    pub fn minimum(&self) -> Result<usize> {
        self.handle.minimum()
    }

    pub fn set_minimum(&mut self, v: usize) -> Result<()> {
        let page = self.page()?.max(1);
        let max = self.max.max(v.saturating_add(page));
        let (min, max) = fit_page(v, max, page);
        self.max = max;
        self.apply_range(min, page)?;
        self.set_pos(self.pos()?)?;
        Ok(())
    }

    pub fn maximum(&self) -> Result<usize> {
        Ok(self.max)
    }

    pub fn set_maximum(&mut self, v: usize) -> Result<()> {
        let page = self.page()?.max(1);
        let old_min = self.minimum()?;
        let min = if v < old_min.saturating_add(page) {
            v.saturating_sub(page)
        } else {
            old_min
        };
        let max = v.max(min.saturating_add(page));
        let (min, max) = fit_page(min, max, page);
        self.max = max;
        self.apply_range(min, page)?;
        self.set_pos(self.pos()?)?;
        Ok(())
    }

    pub fn page(&self) -> Result<usize> {
        Ok(self.page)
    }

    pub fn set_page(&mut self, v: usize) -> Result<()> {
        let page = v.max(1);
        let (min, max) = fit_page(self.minimum()?, self.max, page);
        self.max = max;
        self.page = page;
        self.apply_range(min, page)?;
        self.set_pos(self.pos()?)?;
        Ok(())
    }

    pub fn pos(&self) -> Result<usize> {
        let min = self.minimum()?;
        let max = self.max.saturating_sub(self.page).max(min);
        Ok(self.handle.pos()?.clamp(min, max))
    }

    pub fn set_pos(&mut self, v: usize) -> Result<()> {
        let min = self.minimum()?;
        let max = self.max.saturating_sub(self.page).max(min);
        self.handle.set_pos(v.clamp(min, max))
    }

    /// Apply the scrollable part of the range to the inner slider. The slider
    /// needs a strictly non-empty range, so it may get one extra hidden value
    /// when the page is as large as the content.
    fn apply_range(&mut self, min: usize, page: usize) -> Result<()> {
        let inner_max = self
            .max
            .saturating_sub(page)
            .max(min)
            .max(min.saturating_add(1));
        self.handle.set_minimum(min)?;
        self.handle.set_maximum(inner_max)
    }

    pub async fn wait_change(&self) {
        self.handle.wait_change().await
    }
}

winio_handle::impl_as_widget!(ScrollBar, handle);
