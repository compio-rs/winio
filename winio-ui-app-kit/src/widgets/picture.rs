use inherit_methods_macro::inherit_methods;
use objc2::{MainThreadOnly, rc::Retained};
use objc2_app_kit::{NSImageScaling, NSImageView};
use winio_handle::AsContainer;
use winio_primitive::{Point, Size};

use crate::{Image, Result, catch, widgets::Widget};

#[derive(Debug)]
pub struct Picture {
    handle: Widget,
    view: Retained<NSImageView>,
}

#[inherit_methods(from = "self.handle")]
impl Picture {
    pub fn new(parent: impl AsContainer) -> Result<Self> {
        let parent = parent.as_container();

        catch(|| unsafe {
            let view = NSImageView::new(parent.as_app_kit().mtm());
            view.setImageScaling(NSImageScaling::ScaleProportionallyUpOrDown);
            let handle = Widget::from_nsview(parent, Retained::cast_unchecked(view.clone()))?;

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
        catch(|| {
            if let Some(image) = image {
                let nsimage = image.nsimage(None)?;
                self.view.setImage(Some(&nsimage));
            } else {
                self.view.setImage(None);
            }
            Ok(())
        })
        .flatten()
    }
}

winio_handle::impl_as_widget!(Picture, handle);
