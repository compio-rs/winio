use inherit_methods_macro::inherit_methods;
use objc2::{MainThreadOnly, rc::Retained};
use objc2_ui_kit::{UIImageView, UIViewContentMode};
use winio_handle::AsContainer;
use winio_primitive::{Point, Size};

use crate::{Image, Result, catch, widgets::Widget};

#[derive(Debug)]
pub struct Picture {
    handle: Widget,
    view: Retained<UIImageView>,
}

#[inherit_methods(from = "self.handle")]
impl Picture {
    pub fn new(parent: impl AsContainer) -> Result<Self> {
        let parent = parent.as_container();

        catch(|| unsafe {
            let view = UIImageView::new(parent.as_ui_kit().mtm());
            view.setContentMode(UIViewContentMode::ScaleAspectFit);
            let handle = Widget::from_uiview(parent, Retained::cast_unchecked(view.clone()))?;

            Ok(Self { handle, view })
        })
        .flatten()
    }

    pub fn is_visible(&self) -> Result<bool>;

    pub fn set_visible(&mut self, v: bool) -> Result<()>;

    pub fn preferred_size(&self) -> Result<Size>;

    pub fn loc(&self) -> Result<Point>;

    pub fn set_loc(&mut self, p: Point) -> Result<()>;

    pub fn size(&self) -> Result<Size>;

    pub fn set_size(&mut self, v: Size) -> Result<()>;

    pub fn tooltip(&self) -> Result<String>;

    pub fn set_tooltip(&mut self, s: impl AsRef<str>) -> Result<()>;

    pub fn set_image(&mut self, image: Option<&Image>) -> Result<()> {
        catch(|| self.view.setImage(image.map(|image| image.as_uiimage())))
    }
}

winio_handle::impl_as_widget!(Picture, handle);
