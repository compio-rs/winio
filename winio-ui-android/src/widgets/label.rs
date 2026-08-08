use inherit_methods_macro::inherit_methods;
use jni::{objects::JString, sys::jfloat};
use winio_handle::{AsContainer, impl_as_widget};
use winio_primitive::{Font, HAlign, Point, Size};

use crate::{
    BaseWidget, JCharSequenceExt, Result, current_activity,
    java::android::{
        graphics::{Typeface, typeface},
        view::gravity,
        widget::TextView as ATextView,
    },
    vm_exec,
};

#[derive(Debug)]
pub struct Label {
    inner: BaseWidget<ATextView<'static>>,
}

#[inherit_methods(from = "self.inner")]
impl Label {
    pub fn new(parent: impl AsContainer) -> Result<Self> {
        vm_exec(|env| {
            let act = current_activity(env)?;
            let widget = ATextView::new(env, act)?;
            let inner = BaseWidget::new_with_env(env, parent.as_container(), widget)?;
            inner.set_gravity(env, gravity::CENTER_VERTICAL | gravity::LEFT)?;
            Ok(Self { inner })
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
        vm_exec(move |env| Ok(self.inner.get_text(env)?.try_to_string(env)?))
    }

    pub fn set_text(&mut self, text: impl AsRef<str>) -> Result<()> {
        vm_exec(move |env| {
            let text = env.new_string(&text)?;
            self.inner.set_text(env, text)?;
            Ok(())
        })
    }

    pub fn halign(&self) -> Result<HAlign> {
        let gravity = vm_exec(|env| self.inner.get_gravity(env))?;
        if gravity & gravity::CENTER_HORIZONTAL != 0 {
            Ok(HAlign::Center)
        } else if gravity & gravity::FILL_HORIZONTAL == gravity::FILL_HORIZONTAL {
            Ok(HAlign::Stretch)
        } else if gravity & gravity::RIGHT != 0 {
            Ok(HAlign::Right)
        } else {
            Ok(HAlign::Left)
        }
    }

    pub fn set_halign(&mut self, align: HAlign) -> Result<()> {
        let gravity = match align {
            HAlign::Left => gravity::LEFT,
            HAlign::Center => gravity::CENTER_HORIZONTAL,
            HAlign::Right => gravity::RIGHT,
            HAlign::Stretch => gravity::FILL_HORIZONTAL,
        } | gravity::CENTER_VERTICAL;
        vm_exec(|env| {
            self.inner.set_gravity(env, gravity)?;
            Ok(())
        })
    }

    pub fn font(&self) -> Result<Font> {
        vm_exec(|env| {
            let paint = self.inner.get_paint(env)?;
            let px = paint.get_text_size(env)?;
            let metrics = self
                .inner
                .as_view()
                .get_resources(env)?
                .get_display_metrics(env)?;
            let typeface = paint.get_typeface(env)?;
            let family = self.inner.get_font_family(env)?;
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
        })
    }

    pub fn set_font(&mut self, font: Font) -> Result<()> {
        vm_exec(|env| {
            let mut style = typeface::NORMAL;
            if font.bold {
                style |= typeface::BOLD;
            }
            if font.italic {
                style |= typeface::ITALIC;
            }
            let family = env.new_string(&font.family)?;
            self.inner.set_text_size(env, font.size as jfloat)?;
            let default = Typeface::DEFAULT(env)?;
            self.inner.set_font_family(env, &family)?;
            self.inner.set_typeface_style(env, &default, style)?;
            Ok(())
        })
    }
}

impl_as_widget!(Label, inner);
