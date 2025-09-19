use inherit_methods_macro::inherit_methods;
use objc2::rc::Retained;
use objc2_app_kit::{NSScrollView, NSView};
use objc2_foundation::MainThreadMarker;
use winio_handle::{AsContainer, AsRawContainer, BorrowedContainer, RawContainer};
use winio_primitive::{Point, Size};

use crate::ui::Widget;

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

    pub fn set_size(&mut self, v: Size);

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
