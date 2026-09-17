use inherit_methods_macro::inherit_methods;
use winio_handle::{AsContainer, impl_as_widget};
use winio_primitive::{Point, Size};

use crate::{
    BaseWidget, Image, Result, current_activity,
    java::android::{graphics::drawable::Drawable, widget::ImageView as AImageView},
    vm_exec,
};

#[derive(Debug)]
pub struct Picture {
    inner: BaseWidget<AImageView<'static>>,
}

#[inherit_methods(from = "self.inner")]
impl Picture {
    pub fn new(parent: impl AsContainer) -> Result<Self> {
        vm_exec(|env| {
            let act = current_activity(env)?;
            let widget = AImageView::new(env, act)?;
            let inner = BaseWidget::new_with_env(env, parent.as_container(), widget)?;
            Ok(Self { inner })
        })
    }

    pub fn is_visible(&self) -> Result<bool>;

    pub fn set_visible(&mut self, visible: bool) -> Result<()>;

    pub fn loc(&self) -> Result<Point>;

    pub fn set_loc(&mut self, p: Point) -> Result<()>;

    pub fn size(&self) -> Result<Size>;

    pub fn set_size(&mut self, v: Size) -> Result<()>;

    pub fn preferred_size(&self) -> Result<Size>;

    pub fn tooltip(&self) -> Result<String>;

    pub fn set_tooltip(&mut self, s: impl AsRef<str>) -> Result<()>;

    pub fn set_image(&mut self, image: Option<&Image>) -> Result<()> {
        vm_exec(|env| {
            if let Some(image) = image {
                let drawable = image.drawable(env)?;
                self.inner.set_image_drawable(env, drawable)?;
            } else {
                self.inner.set_image_drawable(env, Drawable::null())?;
            }
            Ok(())
        })
    }
}

impl_as_widget!(Picture, inner);
