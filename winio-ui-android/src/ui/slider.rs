use std::sync::Arc;

use inherit_methods_macro::inherit_methods;
use jni::{objects::JObject, refs::LoaderContext};
use jni_min_helper::DynamicProxy;
use winio_callback::SyncCallback;
use winio_handle::{AsContainer, impl_as_widget};
use winio_primitive::{Orient, Point, Size, TickPosition};

use crate::{AView, BaseWidget, Context, Result, current_activity, vm_exec};

jni::bind_java_type! {
    ASlider => com.google.android.material.slider.Slider,
    type_map {
        AView => android.view.View,
        Context => android.content.Context,
    },
    constructors {
        fn new(&Context),
    },
    methods {
        fn get_value_from() -> jfloat,
        fn set_value_from(from: jfloat),
        fn get_value_to() -> jfloat,
        fn set_value_to(to: jfloat),
        fn get_value() -> jfloat,
        fn set_value(value: jfloat),

        fn get_tick_visibility_mode() -> jint,
        fn set_tick_visibility_mode(mode: jint),
        fn set_orientation(orient: jint),
        fn is_vertical() -> jboolean,
    },
    is_instance_of = {
        view = AView,
    }
}

const TICK_VISIBILITY_AUTO_LIMIT: i32 = 0;
const TICK_VISIBILITY_HIDDEN: i32 = 2;

const HORIZONTAL: i32 = 0;
const VERTICAL: i32 = 1;

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
            let widget = ASlider::new(env, act)?;
            let inner = BaseWidget::new_with_env(env, parent.as_container(), widget)?;
            let on_change = Arc::new(SyncCallback::new());
            let change_proxy = DynamicProxy::build(
                env,
                &LoaderContext::None,
                [jni::jni_str!(
                    "com/google/android/material/slider/Slider$OnChangeListener"
                )],
                {
                    let on_change = on_change.clone();
                    move |_env, _this, _args| {
                        on_change.signal(());
                        Ok(JObject::null())
                    }
                },
            )?;
            env.call_method(
                inner.as_obj(),
                jni::jni_str!("addOnChangeListener"),
                jni::jni_sig!("(Lcom/google/android/material/slider/Slider$OnChangeListener;)V"),
                &[change_proxy.as_ref().into()],
            )?
            .v()?;
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
        vm_exec(|env| self.inner.set_value_to(env, v as _))?;
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
        vm_exec(|env| self.inner.set_value(env, v as _))?;
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
}

#[inherit_methods(from = "self.handle")]
impl ScrollBar {
    pub fn new(parent: impl AsContainer) -> Result<Self> {
        let slider = Slider::new(parent)?;
        Ok(Self { handle: slider })
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

    pub fn maximum(&self) -> Result<usize>;

    pub fn set_maximum(&mut self, v: usize) -> Result<()>;

    pub fn page(&self) -> Result<usize> {
        Ok(1)
    }

    pub fn set_page(&mut self, _v: usize) -> Result<()> {
        Ok(())
    }

    pub fn pos(&self) -> Result<usize>;

    pub fn set_pos(&mut self, v: usize) -> Result<()>;

    pub async fn wait_change(&self) {
        self.handle.wait_change().await
    }
}

winio_handle::impl_as_widget!(ScrollBar, handle);
