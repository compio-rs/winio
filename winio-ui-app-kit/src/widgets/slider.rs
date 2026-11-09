use inherit_methods_macro::inherit_methods;
use objc2::{
    DefinedClass, MainThreadMarker, MainThreadOnly, define_class, msg_send,
    rc::{Allocated, Retained},
    sel,
};
use objc2_app_kit::{NSSlider, NSTickMarkPosition};
use objc2_foundation::NSObject;
use winio_callback::Callback;
use winio_handle::AsContainer;
use winio_primitive::{Orient, Point, Size, TickPosition};

use crate::{GlobalRuntime, Result, Widget, catch};

#[derive(Debug)]
pub struct Slider {
    handle: Widget,
    view: Retained<NSSlider>,
    delegate: Retained<SliderDelegate>,
    freq: usize,
}

#[inherit_methods(from = "self.handle")]
impl Slider {
    pub fn new(parent: impl AsContainer) -> Result<Self> {
        let parent = parent.as_container();
        let mtm = parent.as_app_kit().mtm();

        catch(|| unsafe {
            let view = NSSlider::new(mtm);
            let handle = Widget::from_nsview(parent, Retained::cast_unchecked(view.clone()))?;

            let delegate = SliderDelegate::new(mtm);
            view.setTarget(Some(&delegate));
            view.setAction(Some(sel!(onAction)));

            view.setEnabled(true);
            view.setMinValue(0.0);
            view.setMaxValue(10.0);
            view.setDoubleValue(0.0);

            let mut this = Self {
                handle,
                view,
                delegate,
                freq: 1,
            };
            this.reset_marks()?;
            Ok(this)
        })
        .flatten()
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

    pub fn tick_pos(&self) -> Result<TickPosition> {
        let tpos = catch(|| self.view.tickMarkPosition())?;
        let tpos = match tpos {
            NSTickMarkPosition::Below => TickPosition::BottomRight,
            NSTickMarkPosition::Above => TickPosition::TopLeft,
            _ => TickPosition::None,
        };
        Ok(tpos)
    }

    pub fn set_tick_pos(&mut self, v: TickPosition) -> Result<()> {
        let tpos = match v {
            TickPosition::BottomRight => NSTickMarkPosition::Below,
            _ => NSTickMarkPosition::Above,
        };
        catch(|| {
            self.view.setTickMarkPosition(tpos);
        })
    }

    pub fn orient(&self) -> Result<Orient> {
        let vertical = catch(|| self.view.isVertical())?;
        if vertical {
            Ok(Orient::Vertical)
        } else {
            Ok(Orient::Horizontal)
        }
    }

    pub fn set_orient(&mut self, v: Orient) -> Result<()> {
        catch(|| self.view.setVertical(matches!(v, Orient::Vertical)))
    }

    pub fn minimum(&self) -> Result<usize> {
        catch(|| self.view.minValue() as _)
    }

    pub fn set_minimum(&mut self, v: usize) -> Result<()> {
        let max = self.maximum()?.max(v.saturating_add(1));
        self.apply_range(v, max)
    }

    pub fn maximum(&self) -> Result<usize> {
        catch(|| self.view.maxValue() as _)
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
        self.reset_marks()
    }

    pub fn pos(&self) -> Result<usize> {
        catch(|| self.view.doubleValue() as _)
    }

    pub fn set_pos(&mut self, pos: usize) -> Result<()> {
        let pos = pos.clamp(self.minimum()?, self.maximum()?);
        catch(|| self.view.setDoubleValue(pos as _))
    }

    fn apply_range(&mut self, min: usize, max: usize) -> Result<()> {
        let value = self.pos()?.clamp(min, max);
        let old_max = self.maximum()?;
        catch(|| {
            if min > old_max {
                self.view.setMaxValue(max as _);
                self.view.setMinValue(min as _);
            } else {
                self.view.setMinValue(min as _);
                self.view.setMaxValue(max as _);
            }
            self.view.setDoubleValue(value as _);
        })?;
        self.reset_marks()
    }

    fn reset_marks(&mut self) -> Result<()> {
        let range = self.maximum()?.saturating_sub(self.minimum()?);
        let marks = range / self.freq.max(1);
        catch(|| self.view.setNumberOfTickMarks(marks as _))
    }

    pub async fn wait_change(&self) {
        self.delegate.ivars().action.wait().await;
    }
}

winio_handle::impl_as_widget!(Slider, handle);

#[derive(Debug, Default)]
struct SliderDelegateIvars {
    action: Callback,
}

define_class! {
    #[unsafe(super(NSObject))]
    #[name = "WinioSliderDelegate"]
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
