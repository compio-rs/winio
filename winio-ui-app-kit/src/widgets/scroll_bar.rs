use inherit_methods_macro::inherit_methods;
use objc2::{
    DefinedClass, MainThreadMarker, MainThreadOnly, define_class, msg_send,
    rc::{Allocated, Retained},
    sel,
};
use objc2_app_kit::{NSControlSize, NSScroller, NSScrollerStyle};
use objc2_foundation::{NSPoint, NSRect, NSSize};
use winio_callback::Callback;
use winio_handle::{AsContainer, BorrowedContainer};
use winio_primitive::{Orient, Point, Size, fit_page};

use crate::{GlobalRuntime, Result, catch, widgets::Widget};

#[derive(Debug)]
struct ScrollBarImpl {
    handle: Widget,
    view: Retained<CustomScroller>,
    min: usize,
    max: usize,
    page: usize,
}

#[inherit_methods(from = "self.handle")]
impl ScrollBarImpl {
    pub fn new(parent: impl AsContainer, vertical: bool) -> Result<Self> {
        let parent = parent.as_container();
        let mtm = parent.as_app_kit().mtm();

        catch(|| unsafe {
            let view = CustomScroller::new(
                mtm,
                if vertical {
                    NSRect::new(NSPoint::ZERO, NSSize::new(10.0, 20.0))
                } else {
                    NSRect::new(NSPoint::ZERO, NSSize::new(20.0, 10.0))
                },
            );
            let handle = Widget::from_nsview(parent, Retained::cast_unchecked(view.clone()))?;

            view.setEnabled(true);
            view.setKnobProportion(0.1);
            view.setDoubleValue(0.0);

            view.setContinuous(true);
            view.setTarget(Some(&view));
            view.setAction(Some(sel!(onChange)));

            Ok(Self {
                handle,
                view,
                min: 0,
                max: 10,
                page: 1,
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

    pub fn tooltip(&self) -> Result<String>;

    pub fn set_tooltip(&mut self, s: impl AsRef<str>) -> Result<()>;

    pub fn minimum(&self) -> Result<usize> {
        Ok(self.min)
    }

    pub fn set_minimum(&mut self, v: usize) -> Result<()> {
        let page = self.page.max(1);
        let max = self.max.max(v.saturating_add(page));
        let (min, max) = fit_page(v, max, page);
        self.min = min;
        self.max = max;
        self.sync_page()?;
        self.set_pos(self.pos()?)
    }

    pub fn maximum(&self) -> Result<usize> {
        Ok(self.max)
    }

    pub fn set_maximum(&mut self, v: usize) -> Result<()> {
        let page = self.page.max(1);
        let min = if v < self.min.saturating_add(page) {
            v.saturating_sub(page)
        } else {
            self.min
        };
        let max = v.max(min.saturating_add(page));
        let (min, max) = fit_page(min, max, page);
        self.min = min;
        self.max = max;
        self.sync_page()?;
        self.set_pos(self.pos()?)
    }

    pub fn page(&self) -> Result<usize> {
        Ok(self.page)
    }

    pub fn set_page(&mut self, v: usize) -> Result<()> {
        let page = v.max(1);
        let (min, max) = fit_page(self.min, self.max, page);
        self.min = min;
        self.max = max;
        self.page = page;
        self.sync_page()?;
        self.set_pos(self.pos()?)
    }

    pub fn pos(&self) -> Result<usize> {
        let max = self.max.saturating_sub(self.page).max(self.min);
        if max == self.min {
            return Ok(self.min);
        }
        let pos = catch(|| self.view.doubleValue())?;
        Ok((pos * (max - self.min) as f64) as usize + self.min)
    }

    pub fn set_pos(&mut self, v: usize) -> Result<()> {
        let max = self.max.saturating_sub(self.page).max(self.min);
        let v = v.clamp(self.min, max);
        if max == self.min {
            return catch(|| self.view.setDoubleValue(0.0));
        }
        catch(|| {
            self.view
                .setDoubleValue((v - self.min) as f64 / (max - self.min) as f64)
        })
    }

    fn sync_page(&mut self) -> Result<()> {
        let range = self.max.saturating_sub(self.min);
        let proportion = if range == 0 {
            1.0
        } else {
            (self.page as f64 / range as f64).min(1.0)
        };
        catch(|| self.view.setKnobProportion(proportion))
    }

    pub async fn wait_change(&self) {
        self.view.ivars().on_move.wait().await
    }
}

winio_handle::impl_as_widget!(ScrollBarImpl, handle);

#[derive(Debug, Default)]
struct CustomScrollerIvars {
    on_move: Callback,
}

define_class! {
    #[unsafe(super(NSScroller))]
    #[name = "WinioCustomScroller"]
    #[ivars = CustomScrollerIvars]
    #[thread_kind = MainThreadOnly]
    #[derive(Debug)]
    struct CustomScroller;

    #[allow(non_snake_case)]
    impl CustomScroller {
        #[unsafe(method_id(initWithFrame:))]
        fn initWithFrame(this: Allocated<Self>, frame: NSRect) -> Option<Retained<Self>> {
            let this = this.set_ivars(CustomScrollerIvars::default());
            unsafe { msg_send![super(this), initWithFrame: frame] }
        }

        #[unsafe(method(onChange))]
        unsafe fn onChange(&self) {
            self.ivars().on_move.signal::<GlobalRuntime>(());
        }
    }
}

impl CustomScroller {
    pub fn new(mtm: MainThreadMarker, frame: NSRect) -> Retained<Self> {
        unsafe { msg_send![mtm.alloc::<Self>(), initWithFrame: frame] }
    }
}

#[derive(Debug)]
pub struct ScrollBar {
    handle: ScrollBarImpl,
    vertical: bool,
}

#[inherit_methods(from = "self.handle")]
impl ScrollBar {
    pub fn new(parent: impl AsContainer) -> Result<Self> {
        let handle = ScrollBarImpl::new(&parent, false)?;
        Ok(Self {
            handle,
            vertical: false,
        })
    }

    fn recreate(&mut self, vertical: bool) -> Result<()> {
        let parent = self.handle.handle.parent()?;
        let mut new_handle = ScrollBarImpl::new(BorrowedContainer::app_kit(&parent), vertical)?;
        new_handle.set_visible(self.handle.is_visible()?)?;
        new_handle.set_enabled(self.handle.is_enabled()?)?;
        new_handle.set_loc(self.handle.loc()?)?;
        new_handle.set_size(self.handle.size()?)?;
        new_handle.set_tooltip(self.handle.tooltip()?)?;
        new_handle.set_minimum(self.handle.minimum()?)?;
        new_handle.set_maximum(self.handle.maximum()?)?;
        new_handle.set_page(self.handle.page()?)?;
        new_handle.set_pos(self.handle.pos()?)?;
        self.handle = new_handle;
        Ok(())
    }

    pub fn is_visible(&self) -> Result<bool>;

    pub fn set_visible(&mut self, v: bool) -> Result<()>;

    pub fn is_enabled(&self) -> Result<bool>;

    pub fn set_enabled(&mut self, v: bool) -> Result<()>;

    pub fn preferred_size(&self) -> Result<Size> {
        catch(|| {
            let width = NSScroller::scrollerWidthForControlSize_scrollerStyle(
                NSControlSize::Regular,
                NSScrollerStyle::Overlay,
                self.handle.view.mtm(),
            );
            if self.vertical {
                Size::new(width, 0.0)
            } else {
                Size::new(0.0, width)
            }
        })
    }

    pub fn min_size(&self) -> Result<Size> {
        self.preferred_size()
    }

    pub fn loc(&self) -> Result<Point>;

    pub fn set_loc(&mut self, p: Point) -> Result<()>;

    pub fn size(&self) -> Result<Size>;

    pub fn set_size(&mut self, v: Size) -> Result<()>;

    pub fn tooltip(&self) -> Result<String>;

    pub fn set_tooltip(&mut self, s: impl AsRef<str>) -> Result<()>;

    pub fn orient(&self) -> Result<Orient> {
        if self.vertical {
            Ok(Orient::Vertical)
        } else {
            Ok(Orient::Horizontal)
        }
    }

    pub fn set_orient(&mut self, v: Orient) -> Result<()> {
        let v = matches!(v, Orient::Vertical);
        if self.vertical != v {
            self.recreate(v)?;
            self.vertical = v;
        }
        Ok(())
    }

    pub fn minimum(&self) -> Result<usize>;

    pub fn set_minimum(&mut self, v: usize) -> Result<()>;

    pub fn maximum(&self) -> Result<usize>;

    pub fn set_maximum(&mut self, v: usize) -> Result<()>;

    pub fn page(&self) -> Result<usize>;

    pub fn set_page(&mut self, v: usize) -> Result<()>;

    pub fn pos(&self) -> Result<usize>;

    pub fn set_pos(&mut self, v: usize) -> Result<()>;

    pub async fn wait_change(&self) {
        self.handle.wait_change().await
    }
}

winio_handle::impl_as_widget!(ScrollBar, handle);
