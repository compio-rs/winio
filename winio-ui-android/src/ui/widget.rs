use std::ops::Deref;

use jni::{
    Env,
    objects::JObject,
    refs::{Global, Reference},
};
use winio_handle::{AsContainer, AsWidget, BorrowedContainer, BorrowedWidget};
use winio_primitive::{Point, Size};

use crate::{Context, FrameLayout, Result, vm_exec};

jni::bind_java_type! {
    pub(crate) AView => "android.view.View",
    type_map {
        Context => android.content.Context,
        ViewGroupLayoutParams => "android.view.ViewGroup$LayoutParams",
    },
    constructors {
        fn new(context: &Context),
    },
    methods {
        fn get_x() -> jfloat,
        fn get_y() -> jfloat,
        fn set_x(x: jfloat),
        fn set_y(y: jfloat),
        fn get_width() -> jint,
        fn get_height() -> jint,
        fn set_layout_params(params: &ViewGroupLayoutParams),
        fn measure(width_spec: jint, height_spec: jint),
        fn get_measured_width() -> jint,
        fn get_measured_height() -> jint,
        fn get_minimum_width() -> jint,
        fn get_minimum_height() -> jint,
        fn get_visibility() -> jint,
        fn set_visibility(visibility: jint),
        fn is_enabled() -> jboolean,
        fn set_enabled(enabled: jboolean),
    }
}

jni::bind_java_type! {
    ViewGroupLayoutParams => "android.view.ViewGroup$LayoutParams",
}

jni::bind_java_type! {
    pub(crate) FrameLayoutLayoutParams => "android.widget.FrameLayout$LayoutParams",
    type_map {
        ViewGroupLayoutParams => "android.view.ViewGroup$LayoutParams",
    },
    constructors {
        fn new(width: jint, height: jint),
    },
    is_instance_of = {
        base = ViewGroupLayoutParams,
    }
}

jni::bind_java_type! {
    pub(crate) OnLayoutChangeListener => "android.view.View$OnLayoutChangeListener",
}

#[derive(Debug)]
pub(crate) struct BaseWidget<T>
where
    T: Into<JObject<'static>>
        + AsRef<JObject<'static>>
        + Default
        + Reference
        + Send
        + Sync
        + 'static,
{
    inner: Global<T>,
}

pub(crate) mod gravity {
    pub const CENTER_HORIZONTAL: i32 = 0x1;
    pub const CENTER_VERTICAL: i32 = 0x10;
    pub const FILL_HORIZONTAL: i32 = 0x7;
    pub const LEFT: i32 = 0x3;
    pub const RIGHT: i32 = 0x5;
}

impl<T> BaseWidget<T>
where
    T: Into<JObject<'static>>
        + AsRef<JObject<'static>>
        + AsRef<AView<'static>>
        + Default
        + Reference
        + Send
        + Sync
        + 'static,
{
    pub(crate) fn new_with_env<'any_local, O>(
        env: &mut Env,
        parent: BorrowedContainer,
        widget: O,
    ) -> Result<Self>
    where
        O: Reference<GlobalKind = T> + AsRef<JObject<'any_local>>,
    {
        let widget = env.new_global_ref(widget)?;
        let parent = env.new_local_ref(parent.as_container().to_android())?;
        let parent = unsafe { FrameLayout::from_raw(env, parent.into_raw()) };
        parent.add_view(env, &widget)?;
        Ok(Self { inner: widget })
    }

    pub fn as_obj(&self) -> &JObject<'static> {
        self.inner.as_obj()
    }

    pub fn as_view(&self) -> &AView<'static> {
        self.inner.as_ref()
    }

    pub fn loc(&self) -> Result<Point> {
        vm_exec(move |env| {
            let x = self.as_view().get_x(env)?;
            let y = self.as_view().get_y(env)?;
            Ok(Point::new(x as _, y as _))
        })
    }

    pub fn set_loc(&self, p: Point) -> Result<()> {
        vm_exec(move |env| {
            self.as_view().set_x(env, p.x as f32)?;
            self.as_view().set_y(env, p.y as f32)?;
            Ok(())
        })
    }

    pub fn size(&self) -> Result<Size> {
        vm_exec(|env| {
            let width = self.as_view().get_width(env)?;
            let height = self.as_view().get_height(env)?;
            Ok(Size::new(width as _, height as _))
        })
    }

    pub fn set_size(&self, size: Size) -> Result<()> {
        vm_exec(move |env| {
            let params = FrameLayoutLayoutParams::new(env, size.width as i32, size.height as i32)?;
            self.as_view().set_layout_params(env, params)?;
            Ok(())
        })
    }

    pub fn preferred_size(&self) -> Result<Size> {
        vm_exec(move |env| {
            self.as_view().measure(env, 0, 0)?;
            let width = self.as_view().get_measured_width(env)?;
            let height = self.as_view().get_measured_height(env)?;
            Ok(Size::new(width as _, height as _))
        })
    }

    pub fn min_size(&self) -> Result<Size> {
        vm_exec(move |env| {
            let width = self.as_view().get_minimum_width(env)?;
            let height = self.as_view().get_minimum_height(env)?;
            Ok(Size::new(width as _, height as _))
        })
    }

    pub fn is_visible(&self) -> Result<bool> {
        vm_exec(move |env| {
            let vis = self.as_view().get_visibility(env)?;
            Ok(vis == 0)
        })
    }

    pub fn set_visible(&mut self, visible: bool) -> Result<()> {
        vm_exec(move |env| {
            self.as_view()
                .set_visibility(env, if visible { 0 } else { 4 })?;
            Ok(())
        })
    }

    pub fn tooltip(&self) -> Result<String> {
        Ok(String::new())
    }

    pub fn set_tooltip(&mut self, _s: impl AsRef<str>) -> Result<()> {
        Ok(())
    }

    pub fn is_enabled(&self) -> Result<bool> {
        vm_exec(move |env| Ok(self.as_view().is_enabled(env)?))
    }

    pub fn set_enabled(&mut self, enabled: bool) -> Result<()> {
        vm_exec(move |env| {
            self.as_view().set_enabled(env, enabled)?;
            Ok(())
        })
    }
}

impl<T> From<Global<T>> for BaseWidget<T>
where
    T: Into<JObject<'static>>
        + AsRef<JObject<'static>>
        + Default
        + Reference
        + Send
        + Sync
        + 'static,
{
    fn from(value: Global<T>) -> Self {
        Self { inner: value }
    }
}

impl<T> Deref for BaseWidget<T>
where
    T: Into<JObject<'static>>
        + AsRef<JObject<'static>>
        + Default
        + Reference
        + Send
        + Sync
        + 'static,
{
    type Target = Global<T>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> AsWidget for BaseWidget<T>
where
    T: Into<JObject<'static>>
        + AsRef<JObject<'static>>
        + Default
        + Reference
        + Send
        + Sync
        + 'static,
{
    fn as_widget(&self) -> BorrowedWidget<'_> {
        unsafe { BorrowedWidget::android(self.inner.as_obj()) }
    }
}

impl<T> AsContainer for BaseWidget<T>
where
    T: Into<JObject<'static>>
        + AsRef<JObject<'static>>
        + Default
        + Reference
        + Send
        + Sync
        + 'static,
{
    fn as_container(&self) -> BorrowedContainer<'_> {
        unsafe { BorrowedContainer::android(self.inner.as_obj()) }
    }
}
