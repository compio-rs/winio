use inherit_methods_macro::inherit_methods;
use winio_handle::{AsContainer, impl_as_widget};
use winio_primitive::{HAlign, Point, Size};

use crate::{BaseWidget, Result, vm_exec};

#[derive(Debug)]
pub struct Label {
    inner: BaseWidget,
}

// noinspection SpellCheckingInspection
#[inherit_methods(from = "self.inner")]
impl Label {
    const CENTER_HORIZONTAL: i32 = 0x1;
    const CENTER_VERTICAL: i32 = 0x10;
    const FILL_HORIZONTAL: i32 = 0x7;
    const LEFT: i32 = 0x3;
    const RIGHT: i32 = 0x5;
    const WIDGET_CLASS: &'static str = "android/widget/TextView";

    pub fn new(parent: impl AsContainer) -> Result<Self> {
        vm_exec(|env| {
            let inner = BaseWidget::new_with_env(env, parent.as_container(), Self::WIDGET_CLASS)?;
            env.call_method(
                inner.as_obj(),
                jni::jni_str!("setGravity"),
                jni::jni_sig!("(I)V"),
                &[jni::JValue::Int(Self::CENTER_VERTICAL | Self::LEFT)],
            )?
            .v()?;
            Ok(Self { inner })
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

    pub fn halign(&self) -> Result<HAlign> {
        let gravity = vm_exec(|env| {
            Ok(env
                .call_method(
                    self.inner.as_obj(),
                    jni::jni_str!("getGravity"),
                    jni::jni_sig!("()I"),
                    &[],
                )?
                .i()?)
        })?;
        if gravity & Self::CENTER_HORIZONTAL != 0 {
            Ok(HAlign::Center)
        } else if gravity & Self::FILL_HORIZONTAL == Self::FILL_HORIZONTAL {
            Ok(HAlign::Stretch)
        } else if gravity & Self::RIGHT != 0 {
            Ok(HAlign::Right)
        } else {
            Ok(HAlign::Left)
        }
    }

    pub fn set_halign(&self, align: HAlign) -> Result<()> {
        let gravity = match align {
            HAlign::Left => Self::LEFT,
            HAlign::Center => Self::CENTER_HORIZONTAL,
            HAlign::Right => Self::RIGHT,
            HAlign::Stretch => Self::FILL_HORIZONTAL,
        } | Self::CENTER_VERTICAL;
        vm_exec(|env| {
            env.call_method(
                self.inner.as_obj(),
                jni::jni_str!("setGravity"),
                jni::jni_sig!("(I)V"),
                &[jni::JValue::Int(gravity)],
            )?
            .v()?;
            Ok(())
        })
    }
}

impl From<BaseWidget> for Label {
    fn from(value: BaseWidget) -> Self {
        Self { inner: value }
    }
}

impl_as_widget!(Label, inner);
