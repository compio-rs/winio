use inherit_methods_macro::inherit_methods;
use winio_handle::{AsContainer, impl_as_widget};
use winio_primitive::{HAlign, Point, Size};

use crate::{BaseWidget, Result, gravity, vm_exec};

#[derive(Debug)]
pub struct Label {
    inner: BaseWidget,
}

// noinspection SpellCheckingInspection
#[inherit_methods(from = "self.inner")]
impl Label {
    const WIDGET_CLASS: &'static str = "android/widget/TextView";

    pub fn new(parent: impl AsContainer) -> Result<Self> {
        vm_exec(|env| {
            let inner = BaseWidget::new_with_env(env, parent.as_container(), Self::WIDGET_CLASS)?;
            env.call_method(
                inner.as_obj(),
                jni::jni_str!("setGravity"),
                jni::jni_sig!("(I)V"),
                &[jni::JValue::Int(gravity::CENTER_VERTICAL | gravity::LEFT)],
            )?
            .v()?;
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

    pub fn text(&self) -> Result<String>;

    pub fn set_text(&mut self, text: impl AsRef<str>) -> Result<()>;

    pub fn halign(&self) -> Result<HAlign>;

    pub fn set_halign(&mut self, align: HAlign) -> Result<()>;
}

impl_as_widget!(Label, inner);
