use std::cell::RefCell;

use inherit_methods_macro::inherit_methods;
use objc2::{
    DeclaredClass, MainThreadOnly, define_class, msg_send,
    rc::{Allocated, Retained},
    sel,
};
use objc2_core_foundation::{CGPoint, CGRect, CGSize};
use objc2_foundation::{MainThreadMarker, NSObject, NSString};
use objc2_ui_kit::{UIControlEvents, UISegmentedControl, UIView};
use winio_callback::Callback;
use winio_handle::{AsContainer, BorrowedContainer};
use winio_primitive::{Point, Size};

use crate::{Error, GlobalRuntime, Result, catch, from_cgsize, from_nsstring, widgets::Widget};

const TAB_HEIGHT: f64 = 30.0;

#[derive(Debug)]
pub struct TabView {
    handle: Widget,
    view: Retained<UIView>,
    segment: Retained<UISegmentedControl>,
    delegate: Retained<TabViewDelegate>,
    views: RefCell<Vec<Retained<UIView>>>,
}

#[inherit_methods(from = "self.handle")]
impl TabView {
    pub fn new(parent: impl AsContainer) -> Result<Self> {
        let parent = parent.as_container();
        let mtm = parent.as_ui_kit().mtm();

        catch(|| {
            let view = UIView::new(mtm);
            let handle =
                Widget::from_uiview(parent, unsafe { Retained::cast_unchecked(view.clone()) })?;

            let segment = UISegmentedControl::new(mtm);
            let delegate = TabViewDelegate::new(mtm);

            unsafe {
                segment.addTarget_action_forControlEvents(
                    Some(&delegate),
                    sel!(onSegmentChange),
                    UIControlEvents::ValueChanged,
                );
            }
            view.addSubview(&segment);

            Ok(Self {
                handle,
                view,
                segment,
                delegate,
                views: RefCell::new(Vec::new()),
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

    pub fn set_size(&mut self, mut v: Size) -> Result<()> {
        v.height = v.height.max(TAB_HEIGHT);
        self.handle.set_size(v)?;
        catch(|| {
            self.segment
                .setFrame(CGRect::new(CGPoint::ZERO, CGSize::new(v.width, TAB_HEIGHT)));
            for vw in self.views.borrow().iter() {
                vw.setFrame(CGRect::new(
                    CGPoint::new(0.0, TAB_HEIGHT),
                    CGSize::new(v.width, v.height - TAB_HEIGHT),
                ));
            }
        })
    }

    pub fn selection(&self) -> Result<Option<usize>> {
        catch(|| {
            let index = self.segment.selectedSegmentIndex();
            if index < 0 { None } else { Some(index as _) }
        })
    }

    pub fn set_selection(&mut self, i: usize) -> Result<()> {
        catch(|| {
            self.segment.setSelectedSegmentIndex(i as isize);
            for (idx, vw) in self.views.borrow().iter().enumerate() {
                vw.setHidden(idx != i);
            }
        })
    }

    fn update_visible(&self) {
        let index = self.segment.selectedSegmentIndex();
        if index >= 0 {
            let index = index as usize;
            for (idx, vw) in self.views.borrow().iter().enumerate() {
                vw.setHidden(idx != index);
            }
        }
    }

    pub async fn wait_select(&self) {
        self.delegate.ivars().did_select.wait().await;
        self.update_visible();
    }

    pub fn insert(&mut self, i: usize, item: &TabViewItem) -> Result<()> {
        catch(|| {
            self.segment
                .insertSegmentWithTitle_atIndex_animated(Some(&item.label), i, false);
            let mut size = self.size()?;
            size.height = size.height.max(TAB_HEIGHT);
            item.view.setFrame(CGRect::new(
                CGPoint::new(0.0, TAB_HEIGHT),
                CGSize::new(size.width, size.height - TAB_HEIGHT),
            ));
            item.view.setHidden(true);
            self.view.addSubview(&item.view);
            self.views.borrow_mut().insert(i, item.view.clone());
            Ok(())
        })
        .flatten()?;
        if self.len()? == 1 {
            self.set_selection(0)?;
        }
        Ok(())
    }

    pub fn remove(&mut self, i: usize) -> Result<()> {
        catch(|| {
            self.segment.removeSegmentAtIndex_animated(i, false);
            if let Some(vw) = self.views.borrow().get(i) {
                vw.removeFromSuperview();
            }
            self.views.borrow_mut().remove(i);
        })
    }

    pub fn len(&self) -> Result<usize> {
        catch(|| self.segment.numberOfSegments() as _)
    }

    pub fn is_empty(&self) -> Result<bool> {
        Ok(self.len()? == 0)
    }

    pub fn clear(&mut self) -> Result<()> {
        catch(|| {
            self.segment.removeAllSegments();
            for vw in self.views.borrow().iter() {
                vw.removeFromSuperview();
            }
            self.views.borrow_mut().clear();
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
    #[name = "WinioTabViewDelegateUIKit"]
    #[ivars = TabViewDelegateIvars]
    #[thread_kind = MainThreadOnly]
    #[derive(Debug)]
    struct TabViewDelegate;

    #[allow(non_snake_case)]
    impl TabViewDelegate {
        #[unsafe(method_id(init))]
        fn init(this: Allocated<Self>) -> Option<Retained<Self>> {
            let this = this.set_ivars(TabViewDelegateIvars::default());
            unsafe { msg_send![super(this), init] }
        }

        #[unsafe(method(onSegmentChange))]
        unsafe fn onSegmentChange(&self) {
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
    label: Retained<NSString>,
    view: Retained<UIView>,
}

impl TabViewItem {
    pub fn new() -> Result<Self> {
        let mtm = MainThreadMarker::new().ok_or(Error::NotMainThread)?;
        catch(|| {
            let view = UIView::new(mtm);
            Self {
                label: NSString::from_str(""),
                view,
            }
        })
    }

    pub fn text(&self) -> Result<String> {
        catch(|| from_nsstring(&self.label))
    }

    pub fn set_text(&mut self, s: impl AsRef<str>) -> Result<()> {
        self.label = NSString::from_str(s.as_ref());
        Ok(())
    }

    pub fn size(&self) -> Result<Size> {
        catch(|| from_cgsize(self.view.frame().size))
    }
}

impl AsContainer for TabViewItem {
    fn as_container(&self) -> BorrowedContainer<'_> {
        BorrowedContainer::ui_kit(&self.view)
    }
}
