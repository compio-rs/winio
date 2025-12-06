use inherit_methods_macro::inherit_methods;
use objc2::{
    DefinedClass, MainThreadOnly, define_class, msg_send,
    rc::{Allocated, Retained},
    runtime::ProtocolObject,
};
use objc2_app_kit::{NSTabView, NSTabViewDelegate, NSTabViewItem, NSView};
use objc2_foundation::{MainThreadMarker, NSObject, NSObjectProtocol, NSString};
use winio_callback::Callback;
use winio_handle::{AsContainer, BorrowedContainer};
use winio_primitive::{Point, Size};

use crate::{Error, GlobalRuntime, Result, catch, from_cgsize, from_nsstring, ui::Widget};

#[derive(Debug)]
pub struct TabView {
    handle: Widget,
    view: Retained<NSTabView>,
    delegate: Retained<TabViewDelegate>,
}

#[inherit_methods(from = "self.handle")]
impl TabView {
    pub fn new(parent: impl AsContainer) -> Result<Self> {
        let parent = parent.as_container();
        let mtm = parent.as_app_kit().mtm();

        catch(|| unsafe {
            let view = NSTabView::new(mtm);
            let handle = Widget::from_nsview(parent, Retained::cast_unchecked(view.clone()))?;

            let delegate = TabViewDelegate::new(mtm);
            let del_obj = ProtocolObject::from_ref(&*delegate);
            view.setDelegate(Some(del_obj));

            Ok(Self {
                handle,
                view,
                delegate,
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

    pub fn set_size(&mut self, v: Size) -> Result<()>;

    pub fn selection(&self) -> Result<Option<usize>> {
        catch(|| {
            self.view
                .selectedTabViewItem()
                .map(|item| self.view.indexOfTabViewItem(&item) as _)
        })
    }

    pub fn set_selection(&mut self, i: usize) -> Result<()> {
        catch(|| self.view.selectTabViewItemAtIndex(i as _))
    }

    pub async fn wait_select(&self) {
        self.delegate.ivars().did_select.wait().await
    }

    pub fn insert(&mut self, i: usize, item: &TabViewItem) -> Result<()> {
        catch(|| self.view.insertTabViewItem_atIndex(&item.item, i as _))
    }

    pub fn remove(&mut self, i: usize) -> Result<()> {
        catch(|| {
            self.view
                .removeTabViewItem(&self.view.tabViewItemAtIndex(i as _))
        })
    }

    pub fn len(&self) -> Result<usize> {
        catch(|| self.view.numberOfTabViewItems() as _)
    }

    pub fn is_empty(&self) -> Result<bool> {
        Ok(self.len()? == 0)
    }

    pub fn clear(&mut self) -> Result<()> {
        catch(|| {
            while self.view.numberOfTabViewItems() > 0 {
                self.view
                    .removeTabViewItem(&self.view.tabViewItemAtIndex(0));
            }
        })
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
    view: Retained<NSView>,
}

impl TabViewItem {
    pub fn new() -> Result<Self> {
        let mtm = MainThreadMarker::new().ok_or(Error::NotMainThread)?;
        catch(|| {
            let item = NSTabViewItem::new();
            let view = NSView::new(mtm);
            item.setView(Some(&view));
            Self { item, view }
        })
    }

    pub fn text(&self) -> Result<String> {
        catch(|| from_nsstring(&self.item.label()))
    }

    pub fn set_text(&mut self, s: impl AsRef<str>) -> Result<()> {
        catch(|| self.item.setLabel(&NSString::from_str(s.as_ref())))
    }

    pub fn size(&self) -> Result<Size> {
        catch(|| {
            let frame = self.view.frame().size;
            Ok(from_cgsize(frame))
        })
        .flatten()
    }
}

impl AsContainer for TabViewItem {
    fn as_container(&self) -> BorrowedContainer<'_> {
        BorrowedContainer::app_kit(&self.view)
    }
}
