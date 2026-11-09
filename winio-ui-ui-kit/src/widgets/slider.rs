use inherit_methods_macro::inherit_methods;
use objc2::{
    DeclaredClass, MainThreadMarker, MainThreadOnly, define_class, msg_send,
    rc::{Allocated, Retained},
    runtime::AnyObject,
    sel,
};
use objc2_core_graphics::{CGAffineTransformIdentity, CGAffineTransformMakeRotation};
use objc2_foundation::NSObject;
use objc2_ui_kit::{UIControlEvents, UISlider};
use winio_callback::Callback;
use winio_handle::AsContainer;
use winio_primitive::{Orient, Point, Size, TickPosition, fit_page};

use crate::{GlobalRuntime, Result, Widget, catch};

#[derive(Debug)]
pub struct Slider {
    handle: Widget,
    view: Retained<UISlider>,
    delegate: Retained<SliderDelegate>,
    vertical: bool,
    freq: usize,
}

#[inherit_methods(from = "self.handle")]
impl Slider {
    pub fn new(parent: impl AsContainer) -> Result<Self> {
        let parent = parent.as_container();
        let mtm = parent.as_ui_kit().mtm();

        catch(|| unsafe {
            let view = UISlider::new(mtm);
            let handle = Widget::from_uiview(parent, Retained::cast_unchecked(view.clone()))?;

            let delegate = SliderDelegate::new(mtm);
            let obj: &AnyObject = &delegate;
            view.addTarget_action_forControlEvents(
                Some(obj),
                sel!(onAction),
                UIControlEvents::ValueChanged,
            );

            view.setEnabled(true);
            view.setMinimumValue(0.0);
            view.setMaximumValue(10.0);
            view.setValue(0.0);

            Ok(Self {
                handle,
                view,
                delegate,
                vertical: false,
                freq: 1,
            })
        })
        .flatten()
    }

    pub fn is_visible(&self) -> Result<bool>;

    pub fn set_visible(&mut self, v: bool) -> Result<()>;

    pub fn is_enabled(&self) -> Result<bool>;

    pub fn set_enabled(&mut self, v: bool) -> Result<()>;

    pub fn preferred_size(&self) -> Result<Size> {
        if self.vertical {
            Ok(Size::new(20.0, 0.0))
        } else {
            Ok(Size::new(0.0, 20.0))
        }
    }

    pub fn loc(&self) -> Result<Point>;

    pub fn set_loc(&mut self, p: Point) -> Result<()>;

    pub fn size(&self) -> Result<Size>;

    pub fn set_size(&mut self, v: Size) -> Result<()>;

    pub fn tooltip(&self) -> Result<String>;

    pub fn set_tooltip(&mut self, s: impl AsRef<str>) -> Result<()>;

    pub fn tick_pos(&self) -> Result<TickPosition> {
        Ok(TickPosition::None)
    }

    pub fn set_tick_pos(&mut self, _v: TickPosition) -> Result<()> {
        Ok(())
    }

    pub fn orient(&self) -> Result<Orient> {
        if self.vertical {
            Ok(Orient::Vertical)
        } else {
            Ok(Orient::Horizontal)
        }
    }

    pub fn set_orient(&mut self, v: Orient) -> Result<()> {
        self.vertical = matches!(v, Orient::Vertical);
        if self.vertical {
            self.view
                .setTransform(CGAffineTransformMakeRotation(std::f64::consts::FRAC_PI_2));
        } else {
            self.view.setTransform(unsafe { CGAffineTransformIdentity });
        }
        Ok(())
    }

    pub fn minimum(&self) -> Result<usize> {
        catch(|| self.view.minimumValue() as _)
    }

    pub fn set_minimum(&mut self, v: usize) -> Result<()> {
        let max = self.maximum()?.max(v.saturating_add(1));
        self.apply_range(v, max)
    }

    pub fn maximum(&self) -> Result<usize> {
        catch(|| self.view.maximumValue() as _)
    }

    pub fn set_maximum(&mut self, v: usize) -> Result<()> {
        let old_min = self.minimum()?;
        let min = if v < old_min.saturating_add(1) {
            v.saturating_sub(1)
        } else {
            old_min
        };
        let max = v.max(min.saturating_add(1));
        self.apply_range(min, max)
    }

    pub fn freq(&self) -> Result<usize> {
        Ok(self.freq)
    }

    pub fn set_freq(&mut self, v: usize) -> Result<()> {
        self.freq = v.max(1);
        Ok(())
    }

    pub fn pos(&self) -> Result<usize> {
        catch(|| self.view.value() as _)
    }

    pub fn set_pos(&mut self, pos: usize) -> Result<()> {
        catch(|| self.view.setValue(pos as _))
    }

    fn apply_range(&mut self, min: usize, max: usize) -> Result<()> {
        let old_max = self.maximum()?;
        if min > old_max {
            catch(|| {
                self.view.setMaximumValue(max as _);
                self.view.setMinimumValue(min as _);
            })?;
        } else {
            catch(|| {
                self.view.setMinimumValue(min as _);
                self.view.setMaximumValue(max as _);
            })?;
        }
        Ok(())
    }

    pub async fn wait_change(&self) {
        self.delegate.ivars().action.wait().await;
    }
}

winio_handle::impl_as_widget!(Slider, handle);

#[derive(Debug)]
pub struct ScrollBar {
    handle: Slider,
    page: usize,
    /// The public content extent. The inner slider stores `max - page`, and
    /// when the page fills the range it is set to `min + 1` so the slider has
    /// a non-empty range; the public maximum is kept separately.
    max: usize,
}

#[inherit_methods(from = "self.handle")]
impl ScrollBar {
    pub fn new(parent: impl AsContainer) -> Result<Self> {
        let mut slider = Slider::new(parent)?;
        slider.set_maximum(9)?;
        Ok(Self {
            handle: slider,
            page: 1,
            max: 10,
        })
    }

    pub fn is_visible(&self) -> Result<bool>;

    pub fn set_visible(&mut self, v: bool) -> Result<()>;

    pub fn is_enabled(&self) -> Result<bool>;

    pub fn set_enabled(&mut self, v: bool) -> Result<()>;

    pub fn preferred_size(&self) -> Result<Size>;

    pub fn loc(&self) -> Result<Point>;

    pub fn set_loc(&mut self, p: Point) -> Result<()>;

    pub fn size(&self) -> Result<Size>;

    pub fn set_size(&mut self, v: Size) -> Result<()>;

    pub fn tooltip(&self) -> Result<String>;

    pub fn set_tooltip(&mut self, s: impl AsRef<str>) -> Result<()>;

    pub fn orient(&self) -> Result<Orient>;

    pub fn set_orient(&mut self, v: Orient) -> Result<()>;

    pub fn minimum(&self) -> Result<usize> {
        self.handle.minimum()
    }

    pub fn set_minimum(&mut self, v: usize) -> Result<()> {
        let page = self.page.max(1);
        let max = self.max.max(v.saturating_add(page));
        let (min, max) = fit_page(v, max, page);
        self.max = max;
        self.apply_range(min, page)?;
        self.set_pos(self.pos()?)?;
        Ok(())
    }

    pub fn maximum(&self) -> Result<usize> {
        Ok(self.max)
    }

    pub fn set_maximum(&mut self, v: usize) -> Result<()> {
        let page = self.page.max(1);
        let old_min = self.minimum()?;
        let min = if v < old_min.saturating_add(page) {
            v.saturating_sub(page)
        } else {
            old_min
        };
        let max = v.max(min.saturating_add(page));
        let (min, max) = fit_page(min, max, page);
        self.max = max;
        self.apply_range(min, page)?;
        self.set_pos(self.pos()?)?;
        Ok(())
    }

    pub fn page(&self) -> Result<usize> {
        Ok(self.page)
    }

    pub fn set_page(&mut self, v: usize) -> Result<()> {
        let page = v.max(1);
        let (min, max) = fit_page(self.minimum()?, self.max, page);
        self.max = max;
        self.page = page;
        self.apply_range(min, page)?;
        self.set_pos(self.pos()?)?;
        Ok(())
    }

    pub fn pos(&self) -> Result<usize> {
        let min = self.minimum()?;
        let max = self.max.saturating_sub(self.page).max(min);
        Ok(self.handle.pos()?.clamp(min, max))
    }

    pub fn set_pos(&mut self, v: usize) -> Result<()> {
        let min = self.minimum()?;
        let max = self.max.saturating_sub(self.page).max(min);
        self.handle.set_pos(v.clamp(min, max))
    }

    fn apply_range(&mut self, min: usize, page: usize) -> Result<()> {
        let inner_max = self
            .max
            .saturating_sub(page)
            .max(min)
            .max(min.saturating_add(1));
        self.handle.set_minimum(min)?;
        self.handle.set_maximum(inner_max)
    }

    pub async fn wait_change(&self) {
        self.handle.wait_change().await
    }
}

winio_handle::impl_as_widget!(ScrollBar, handle);

#[derive(Debug, Default)]
struct SliderDelegateIvars {
    action: Callback,
}

define_class! {
    #[unsafe(super(NSObject))]
    #[name = "WinioSliderDelegateUIKit"]
    #[ivars = SliderDelegateIvars]
    #[thread_kind = MainThreadOnly]
    #[derive(Debug)]
    struct SliderDelegate;

    #[allow(non_snake_case)]
    impl SliderDelegate {
        #[unsafe(method_id(init))]
        fn init(this: Allocated<Self>) -> Option<Retained<Self>> {
            let this = this.set_ivars(SliderDelegateIvars::default());
            unsafe { msg_send![super(this), init] }
        }

        #[unsafe(method(onAction))]
        unsafe fn onAction(&self) {
            self.ivars().action.signal::<GlobalRuntime>(());
        }
    }
}

impl SliderDelegate {
    pub fn new(mtm: MainThreadMarker) -> Retained<Self> {
        unsafe { msg_send![mtm.alloc::<Self>(), init] }
    }
}
