use inherit_methods_macro::inherit_methods;
use objc2::{MainThreadOnly, rc::Retained};
use objc2_core_foundation::CGSize;
use objc2_ui_kit::{UIScrollView, UIView};
use winio_handle::{AsContainer, BorrowedContainer};
use winio_primitive::{Point, Size};

use crate::{Result, catch, from_cgrect, to_cgrect, widgets::Widget};

#[derive(Debug)]
pub struct ScrollView {
    handle: Widget,
    view: Retained<UIScrollView>,
    inner_view: Retained<UIView>,
}

#[inherit_methods(from = "self.handle")]
impl ScrollView {
    pub fn new(parent: impl AsContainer) -> Result<Self> {
        let parent = parent.as_container();
        let mtm = parent.as_ui_kit().mtm();

        catch(|| {
            let view = UIScrollView::new(mtm);
            let inner_view = unsafe { Retained::cast_unchecked::<UIView>(view.clone()) };
            view.setScrollEnabled(true);
            let handle = Widget::from_uiview(parent, inner_view.clone())?;

            Ok(Self {
                handle,
                view,
                inner_view,
            })
        })
        .flatten()
    }

    pub fn is_visible(&self) -> Result<bool>;

    pub fn set_visible(&mut self, v: bool) -> Result<()>;

    pub fn is_enabled(&self) -> Result<bool>;

    pub fn set_enabled(&mut self, v: bool) -> Result<()>;

    pub fn loc(&self) -> Result<Point>;

    pub fn set_loc(&mut self, p: Point) -> Result<()>;

    pub fn size(&self) -> Result<Size>;

    pub fn set_size(&mut self, v: Size) -> Result<()> {
        self.handle.set_size(v)?;
        catch(|| {
            let subviews = self.inner_view.subviews();
            let frames = subviews
                .iter()
                .map(|c| {
                    let mut frame = c.frame();
                    let size = c.sizeThatFits(CGSize::ZERO);
                    frame.size.width = frame.size.width.max(size.width);
                    frame.size.height = frame.size.height.max(size.height);
                    from_cgrect(frame)
                })
                .collect::<Vec<_>>();
            let rect = frames
                .iter()
                .copied()
                .reduce(|a, b| a.union(&b))
                .unwrap_or_default();
            let mut rect = rect.to_box2d();
            rect.min = rect.min.min(Point::zero());
            let mut rect = rect.to_rect();
            if rect.height() < v.height {
                rect.size.height = v.height;
            }
            let frame = to_cgrect(rect);
            self.view.setContentSize(frame.size);
        })
    }

    pub fn hscroll(&self) -> Result<bool> {
        catch(|| self.view.showsHorizontalScrollIndicator())
    }

    pub fn set_hscroll(&mut self, v: bool) -> Result<()> {
        catch(|| self.view.setShowsHorizontalScrollIndicator(v))
    }

    pub fn vscroll(&self) -> Result<bool> {
        catch(|| self.view.showsVerticalScrollIndicator())
    }

    pub fn set_vscroll(&mut self, v: bool) -> Result<()> {
        catch(|| self.view.setShowsVerticalScrollIndicator(v))
    }

    pub async fn start(&self) -> ! {
        std::future::pending().await
    }
}

winio_handle::impl_as_widget!(ScrollView, handle);

impl AsContainer for ScrollView {
    fn as_container(&self) -> BorrowedContainer<'_> {
        BorrowedContainer::ui_kit(&self.inner_view)
    }
}
