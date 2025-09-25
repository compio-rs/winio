use inherit_methods_macro::inherit_methods;
use objc2::{
    DefinedClass, MainThreadOnly, define_class, msg_send,
    rc::{Allocated, Retained},
    runtime::ProtocolObject,
};
use objc2_app_kit::{NSTabView, NSTabViewDelegate, NSTabViewItem, NSView};
use objc2_foundation::{MainThreadMarker, NSObject, NSObjectProtocol, NSString};
use winio_callback::Callback;
use winio_handle::{AsContainer, AsRawContainer, RawContainer};
use winio_primitive::{Point, Size};

use crate::{GlobalRuntime, from_cgsize, from_nsstring, ui::Widget};

#[derive(Debug)]
pub struct TabView {
    handle: Widget,
    view: Retained<NSTabView>,
    delegate: Retained<TabViewDelegate>,
}

#[inherit_methods(from = "self.handle")]
impl TabView {
    pub fn new(parent: impl AsContainer) -> Self {
        unsafe {
            let parent = parent.as_container();
            let mtm = parent.mtm();

            let view = NSTabView::new(mtm);
            let handle = Widget::from_nsview(parent, Retained::cast_unchecked(view.clone()));

            let delegate = TabViewDelegate::new(mtm);
            let del_obj = ProtocolObject::from_ref(&*delegate);
            view.setDelegate(Some(del_obj));

            Self {
                handle,
                view,
                delegate,
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

    pub fn selection(&self) -> Option<usize> {
        unsafe {
            self.view
                .selectedTabViewItem()
                .map(|item| self.view.indexOfTabViewItem(&item) as _)
        }
    }

    pub fn set_selection(&mut self, i: Option<usize>) {
        unsafe {
            if let Some(i) = i {
                self.view.selectTabViewItemAtIndex(i as _);
            }
        }
    }

    pub async fn wait_select(&self) {
        self.delegate.ivars().did_select.wait().await
    }

    pub fn insert(&mut self, i: usize, item: &TabViewItem) {
        unsafe {
            self.view.insertTabViewItem_atIndex(&item.item, i as _);
        }
    }

    pub fn remove(&mut self, i: usize) {
        unsafe {
            self.view
                .removeTabViewItem(&self.view.tabViewItemAtIndex(i as _));
        }
    }

    pub fn len(&self) -> usize {
        unsafe { self.view.numberOfTabViewItems() as _ }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn clear(&mut self) {
        while !self.is_empty() {
            self.remove(0);
        }
    }
}

winio_handle::impl_as_widget!(TabView, handle);

#[derive(Debug, Default)]
struct TabViewDelegateIvars {
    did_select: Callback,
}

define_class! {
    #[unsafe(super(NSObject))]
    #[name = "WinioTabViewDelegate"]
    #[ivars = TabViewDelegateIvars]
    #[thread_kind = MainThreadOnly]
    #[derive(Debug)]
    struct TabViewDelegate;

    impl TabViewDelegate {
        #[unsafe(method_id(init))]
        fn init(this: Allocated<Self>) -> Option<Retained<Self>> {
            let this = this.set_ivars(TabViewDelegateIvars::default());
            unsafe { msg_send![super(this), init] }
        }
    }

    unsafe impl NSObjectProtocol for TabViewDelegate {}

    #[allow(non_snake_case)]
    unsafe impl NSTabViewDelegate for TabViewDelegate {
        #[unsafe(method(tabView:didSelectTabViewItem:))]
        unsafe fn tabView_didSelectTabViewItem(
            &self,
            _tab_view: &NSTabView,
            _tab_view_item: Option<&NSTabViewItem>,
        ) {
            self.ivars().did_select.signal::<GlobalRuntime>(());
        }
    }
}

impl TabViewDelegate {
    pub fn new(mtm: MainThreadMarker) -> Retained<Self> {
        unsafe { msg_send![mtm.alloc::<Self>(), init] }
    }
}

#[derive(Debug)]
pub struct TabViewItem {
    item: Retained<NSTabViewItem>,
    mtm: MainThreadMarker,
}

impl TabViewItem {
    pub fn new(parent: &TabView) -> Self {
        unsafe {
            let item = NSTabViewItem::new();
            let mtm = parent.view.mtm();
            item.setView(Some(&NSView::new(mtm)));
            Self { item, mtm }
        }
    }

    pub fn text(&self) -> String {
        unsafe { from_nsstring(&self.item.label()) }
    }

    pub fn set_text(&mut self, s: impl AsRef<str>) {
        unsafe {
            self.item.setLabel(&NSString::from_str(s.as_ref()));
        }
    }

    pub fn size(&self) -> Size {
        unsafe {
            let frame = self.item.view(self.mtm).unwrap().frame().size;
            from_cgsize(frame)
        }
    }
}

impl AsRawContainer for TabViewItem {
    fn as_raw_container(&self) -> RawContainer {
        unsafe { self.item.view(self.mtm).unwrap().clone() }
    }
}

winio_handle::impl_as_container!(TabViewItem);
