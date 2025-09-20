use inherit_methods_macro::inherit_methods;
use objc2::rc::Retained;
use objc2_app_kit::{NSControl, NSScrollView, NSView};
use objc2_foundation::{MainThreadMarker, NSSize};
use winio_handle::{AsContainer, AsRawContainer, BorrowedContainer, RawContainer};
use winio_primitive::{Point, Size};

use crate::{from_cgsize, transform_cgrect, transform_rect, ui::Widget};

#[derive(Debug)]
pub struct ScrollView {
    handle: Widget,
    view: Retained<NSScrollView>,
    inner_view: Retained<NSView>,
}

#[inherit_methods(from = "self.handle")]
impl ScrollView {
    pub fn new(parent: impl AsContainer) -> Self {
        unsafe {
            let mtm = MainThreadMarker::new().unwrap();

            let view = NSScrollView::new(mtm);
            let inner_view = NSView::new(mtm);
            let handle = Widget::from_nsview(parent, Retained::cast_unchecked(view.clone()));

            view.setHasVerticalScroller(true);
            view.setHasHorizontalScroller(true);
            view.setDocumentView(Some(&inner_view));

            Self {
                handle,
                view,
                inner_view,
            }
        }
    }

    pub fn is_visible(&self) -> bool;

    pub fn set_visible(&mut self, v: bool);

    pub fn is_enabled(&self) -> bool;

    pub fn set_enabled(&mut self, v: bool);

    pub fn loc(&self) -> Point;

    pub fn set_loc(&mut self, p: Point);

    pub fn size(&self) -> Size;

    pub fn set_size(&mut self, v: Size) {
        self.handle.set_size(v);
        unsafe {
            let inner_size = from_cgsize(self.inner_view.frame().size);
            let subviews = self.inner_view.subviews();
            let frames = subviews
                .iter()
                .map(|c| {
                    let mut frame = c.frame();
                    if let Ok(c) = c.downcast::<NSControl>() {
                        let size = c.sizeThatFits(NSSize::ZERO);
                        frame.size.width = frame.size.width.max(size.width);
                        frame.size.height = frame.size.height.max(size.height);
                    }
                    transform_cgrect(inner_size, frame)
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
            let frame = transform_rect(v, rect);
            self.inner_view.setFrame(frame);
            let inner_size = from_cgsize(frame.size);
            for (c, rect) in subviews.into_iter().zip(frames) {
                c.setFrame(transform_rect(inner_size, rect));
            }
        }
    }

    pub fn hscroll(&self) -> bool {
        unsafe { self.view.hasHorizontalScroller() }
    }

    pub fn set_hscroll(&mut self, v: bool) {
        unsafe {
            self.view.setHasHorizontalScroller(v);
        }
    }

    pub fn vscroll(&self) -> bool {
        unsafe { self.view.hasVerticalScroller() }
    }

    pub fn set_vscroll(&mut self, v: bool) {
        unsafe {
            self.view.setHasVerticalScroller(v);
        }
    }

    pub async fn start(&self) -> ! {
        std::future::pending().await
    }
}

winio_handle::impl_as_widget!(ScrollView, handle);

impl AsRawContainer for ScrollView {
    fn as_raw_container(&self) -> RawContainer {
        self.inner_view.clone()
    }
}

impl AsContainer for ScrollView {
    fn as_container(&self) -> BorrowedContainer<'_> {
        unsafe { BorrowedContainer::borrow_raw(self.as_raw_container()) }
    }
}
