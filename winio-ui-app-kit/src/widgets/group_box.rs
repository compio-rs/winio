use inherit_methods_macro::inherit_methods;
use objc2::{MainThreadOnly, rc::Retained};
use objc2_app_kit::{NSBox, NSBoxType, NSTitlePosition, NSView};
use objc2_foundation::{NSPoint, NSRect, NSString};
use winio_handle::{AsContainer, BorrowedContainer};
use winio_primitive::{Point, Size};

use crate::{Result, catch, from_cgsize, from_nsstring, to_cgsize, widgets::Widget};

#[derive(Debug)]
pub struct GroupBox {
    handle: Widget,
    view: Retained<NSBox>,
    inner_view: Retained<NSView>,
}

#[inherit_methods(from = "self.handle")]
impl GroupBox {
    pub fn new(parent: impl AsContainer) -> Result<Self> {
        let parent = parent.as_container();
        let mtm = parent.as_app_kit().mtm();

        catch(|| unsafe {
            let view = NSBox::new(mtm);
            view.setBoxType(NSBoxType::Primary);
            view.setTitlePosition(NSTitlePosition::AtTop);

            let inner_view = NSView::new(mtm);
            inner_view.setClipsToBounds(true);
            view.setContentView(Some(&inner_view));

            let handle = Widget::from_nsview(parent, Retained::cast_unchecked(view.clone()))?;

            let this = Self {
                handle,
                view,
                inner_view,
            };
            this.sync_inner_view()?;
            Ok(this)
        })
        .flatten()
    }

    pub fn is_visible(&self) -> Result<bool>;

    pub fn set_visible(&mut self, v: bool) -> Result<()>;

    pub fn loc(&self) -> Result<Point>;

    pub fn set_loc(&mut self, p: Point) -> Result<()>;

    pub fn size(&self) -> Result<Size>;

    pub fn set_size(&mut self, v: Size) -> Result<()> {
        self.handle.set_size(v)?;
        self.sync_inner_view()?;
        Ok(())
    }

    pub fn preferred_size(&self) -> Result<Size>;

    pub fn client_size(&self) -> Result<Size> {
        self.sync_inner_view()
    }

    pub fn text(&self) -> Result<String> {
        catch(|| from_nsstring(&self.view.title()))
    }

    pub fn set_text(&mut self, s: impl AsRef<str>) -> Result<()> {
        catch(|| self.view.setTitle(&NSString::from_str(s.as_ref())))
    }

    fn sync_inner_view(&self) -> Result<Size> {
        catch(|| {
            let border = self.view.borderRect();
            let margins = self.view.contentViewMargins();
            let size = Size::new(
                (border.size.width - margins.width * 2.0).max(0.0),
                (border.size.height - margins.height * 2.0).max(0.0),
            );
            if from_cgsize(self.inner_view.frame().size) != size {
                self.inner_view.setFrame(NSRect::new(
                    NSPoint::new(margins.width, margins.height),
                    to_cgsize(size),
                ));
            }
            Ok(size)
        })
        .flatten()
    }
}

winio_handle::impl_as_widget!(GroupBox, handle);

impl AsContainer for GroupBox {
    fn as_container(&self) -> BorrowedContainer<'_> {
        BorrowedContainer::app_kit(&self.inner_view)
    }
}
