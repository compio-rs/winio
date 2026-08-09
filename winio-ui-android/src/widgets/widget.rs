use std::ops::Deref;

use compio_log::error;
use jni::{
    Env,
    objects::{JObject, JString},
    refs::{Global, Reference},
};
use winio_handle::{AsContainer, AsWidget, BorrowedContainer, BorrowedWidget};
use winio_primitive::{Font, Point, Size};

use crate::{
    Result,
    java::android::{
        graphics::{Typeface, Typeface2, typeface},
        view::{View as AView, ViewGroup as AViewGroup, gravity},
        widget::{FrameLayout, FrameLayoutLayoutParams, TextView as ATextView},
    },
    platform::dpi::{logical_point, logical_size, physical_point, physical_size},
    vm_exec,
};

/// Read the font of a [`ATextView`].
pub(crate) fn text_view_to_font(
    env: &mut Env<'_>,
    view: &BaseWidget<ATextView<'_>>,
) -> Result<Font> {
    let px = view.get_text_size(env)?;
    let metrics = view
        .as_view()
        .get_resources(env)?
        .get_display_metrics(env)?;
    let typeface = view.get_typeface(env)?;
    let family = {
        let typeface = env.new_local_ref(&typeface)?;
        let typeface = unsafe { Typeface2::from_raw(env, typeface.into_raw()) };
        typeface
            .get_system_font_family_name(env)
            .unwrap_or_else(|_| JString::null())
    };
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
}

/// Set the font of a [`ATextView`].
pub(crate) fn font_to_text_view(
    env: &mut Env<'_>,
    view: &BaseWidget<ATextView<'_>>,
    font: &Font,
) -> Result<()> {
    let mut style = typeface::NORMAL;
    if font.bold {
        style |= typeface::BOLD;
    }
    if font.italic {
        style |= typeface::ITALIC;
    }
    let family = env.new_string(&font.family)?;
    let typeface = Typeface::create(env, &family, style)?;
    view.set_text_size(env, font.size as _)?;
    view.set_typeface(env, &typeface)?;
    Ok(())
}

#[derive(Debug)]
pub(crate) struct BaseWidget<T>
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
    inner: Global<T>,
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
        parent.as_view_group().add_view(env, &widget)?;
        Ok(Self { inner: widget })
    }

    pub fn as_view(&self) -> &AView<'static> {
        self.inner.as_ref()
    }

    pub fn loc(&self) -> Result<Point> {
        let (x, y) = vm_exec(move |env| {
            let x = self.as_view().get_x(env)?;
            let y = self.as_view().get_y(env)?;
            Result::Ok((x, y))
        })?;
        logical_point(x, y)
    }

    pub fn set_loc(&self, p: Point) -> Result<()> {
        let (x, y) = physical_point(p)?;
        vm_exec(move |env| {
            let params = self.as_view().get_layout_params(env)?;
            let width = params.width(env)?;
            let height = params.height(env)?;
            let params = FrameLayoutLayoutParams::new(env, width, height)?;
            params.as_margin().set_left_margin(env, x as i32)?;
            params.as_margin().set_top_margin(env, y as i32)?;
            params.set_gravity(env, gravity::LEFT | gravity::TOP)?;
            self.as_view().set_layout_params(env, params)?;
            Ok(())
        })
    }

    pub fn size(&self) -> Result<Size> {
        let (width, height) = vm_exec(|env| {
            let width = self.as_view().get_width(env)?;
            let height = self.as_view().get_height(env)?;
            Result::Ok((width as _, height as _))
        })?;
        logical_size(width, height)
    }

    pub fn set_size(&self, size: Size) -> Result<()> {
        let (width, height) = physical_size(size)?;
        vm_exec(move |env| {
            let params = self.as_view().get_layout_params(env)?;
            let params = if env.is_instance_of(&params, FrameLayoutLayoutParams::class_name())? {
                let params = unsafe { FrameLayoutLayoutParams::from_raw(env, params.into_raw()) };
                params.as_base().set_width(env, width as i32)?;
                params.as_base().set_height(env, height as i32)?;
                params
            } else {
                FrameLayoutLayoutParams::new(env, width as i32, height as i32)?
            };
            self.as_view().set_layout_params(env, params)?;
            Ok(())
        })
    }

    pub(crate) fn set_wrap_content(&self) -> Result<()> {
        vm_exec(move |env| {
            let params = FrameLayoutLayoutParams::new(env, -2, -2)?;
            self.as_view().set_layout_params(env, params)?;
            Ok(())
        })
    }

    pub fn preferred_size(&self) -> Result<Size> {
        let (width, height) = vm_exec(move |env| {
            self.as_view().measure(env, 0, 0)?;
            let width = self.as_view().get_measured_width(env)?;
            let height = self.as_view().get_measured_height(env)?;
            Result::Ok((width as f32, height as f32))
        })?;
        // A little hack to make the preferred size a little bigger than the measured
        // size, so that the widget is not too small.
        logical_size(width + 4.0, height)
    }

    pub fn min_size(&self) -> Result<Size> {
        let (width, height) = vm_exec(move |env| {
            let width = self.as_view().get_minimum_width(env)?;
            let height = self.as_view().get_minimum_height(env)?;
            Result::Ok((width as _, height as _))
        })?;
        logical_size(width, height)
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

impl<F> Drop for BaseWidget<F>
where
    F: Into<JObject<'static>>
        + AsRef<JObject<'static>>
        + AsRef<AView<'static>>
        + Default
        + Reference
        + Send
        + Sync
        + 'static,
{
    fn drop(&mut self) {
        let res = vm_exec(|env| {
            let inner = self.as_view();
            let parent = inner.get_parent(env)?;
            if !parent.is_null() {
                let parent = unsafe { AViewGroup::from_raw(env, parent.into_raw()) };
                parent.remove_view(env, inner)?;
            }
            Result::Ok(())
        });
        if let Err(e) = res {
            error!("Failed to remove view from parent: {:?}", e);
        }
    }
}

impl<T> From<Global<T>> for BaseWidget<T>
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
    fn from(value: Global<T>) -> Self {
        Self { inner: value }
    }
}

impl<T> Deref for BaseWidget<T>
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
    type Target = Global<T>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> AsWidget for BaseWidget<T>
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
    fn as_widget(&self) -> BorrowedWidget<'_> {
        unsafe { BorrowedWidget::android(self.inner.as_obj()) }
    }
}

impl<T> AsContainer for BaseWidget<T>
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
    fn as_container(&self) -> BorrowedContainer<'_> {
        unsafe { BorrowedContainer::android(self.inner.as_obj()) }
    }
}
