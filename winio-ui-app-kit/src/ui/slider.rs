use inherit_methods_macro::inherit_methods;
use objc2::{
    DefinedClass, MainThreadMarker, MainThreadOnly, define_class, msg_send,
    rc::{Allocated, Retained},
    sel,
};
use objc2_app_kit::NSSlider;
use objc2_foundation::{NSObject, NSString};
use winio_callback::Callback;
use winio_handle::AsContainer;
use winio_primitive::{Orient, Point, Size};

use crate::{GlobalRuntime, Widget};

#[derive(Debug)]
pub struct Slider {
    handle: Widget,
    view: Retained<NSSlider>,
    delegate: Retained<SliderDelegate>,
}

#[inherit_methods(from = "self.handle")]
impl Slider {
    pub fn new(parent: impl AsContainer) -> Self {
        unsafe {
            let parent = parent.as_container();
            let mtm = parent.mtm();

            let view = NSSlider::new(mtm);
            let handle = Widget::from_nsview(parent, Retained::cast_unchecked(view.clone()));

            let delegate = SliderDelegate::new(mtm);
            view.setTarget(Some(&delegate));
            view.setAction(Some(sel!(onAction)));

            view.setEnabled(true);

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

    pub fn preferred_size(&self) -> Size;

    pub fn loc(&self) -> Point;

    pub fn set_loc(&mut self, p: Point);

    pub fn size(&self) -> Size;

    pub fn set_size(&mut self, v: Size);

    pub fn tooltip(&self) -> String;

    pub fn set_tooltip(&mut self, s: impl AsRef<str>);

    pub fn orient(&self) -> Orient {
        let vertical: bool = unsafe { msg_send![&*self.view, isVertical] };
        if vertical {
            Orient::Vertical
        } else {
            Orient::Horizontal
        }
    }

    pub fn set_orient(&mut self, v: Orient) {
        unsafe {
            self.view.setVertical(matches!(v, Orient::Vertical));
        }
    }

    pub fn minimum(&self) -> usize {
        unsafe { self.view.minValue() as _ }
    }

    pub fn set_minimum(&mut self, v: usize) {
        unsafe {
            self.view.setMinValue(v as _);
        }
    }

    pub fn maximum(&self) -> usize {
        unsafe { self.view.maxValue() as _ }
    }

    pub fn set_maximum(&mut self, v: usize) {
        unsafe {
            self.view.setMaxValue(v as _);
        }
    }

    pub fn freq(&self) -> usize {
        let nmarks = unsafe { self.view.numberOfTickMarks() } as usize;
        let range = self.maximum() - self.minimum();
        range / nmarks
    }

    pub fn set_freq(&mut self, v: usize) {
        unsafe {
            let range = self.maximum() - self.minimum();
            self.view.setNumberOfTickMarks((range / v) as _);
        }
    }

    pub fn pos(&self) -> usize {
        unsafe { self.view.doubleValue() as _ }
    }

    pub fn set_pos(&mut self, pos: usize) {
        unsafe {
            self.view.setDoubleValue(pos as _);
        }
        self.reset_tooltip();
    }

    fn reset_tooltip(&self) {
        unsafe {
            self.view
                .setToolTip(Some(&NSString::from_str(&self.pos().to_string())))
        }
    }

    pub async fn wait_change(&self) {
        self.delegate.ivars().action.wait().await;
        self.reset_tooltip();
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
