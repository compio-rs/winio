use inherit_methods_macro::inherit_methods;
use objc2::{
    DeclaredClass, MainThreadMarker, MainThreadOnly, define_class, msg_send,
    rc::{Allocated, Retained},
    runtime::AnyObject,
    sel,
};
use objc2_foundation::NSObject;
use objc2_ui_kit::{UIControlEvents, UISlider};
use winio_callback::Callback;
use winio_handle::AsContainer;
use winio_primitive::{Orient, Point, Size, TickPosition};

use crate::{GlobalRuntime, Result, Widget, catch};

#[derive(Debug)]
pub struct Slider {
    handle: Widget,
    view: Retained<UISlider>,
    delegate: Retained<SliderDelegate>,
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

    pub fn preferred_size(&self) -> Result<Size> {
        Ok(Size::new(0.0, 30.0))
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
        Ok(Orient::Horizontal)
    }

    pub fn set_orient(&mut self, _v: Orient) -> Result<()> {
        Ok(())
    }

    pub fn minimum(&self) -> Result<usize> {
        catch(|| self.view.minimumValue() as _)
    }

    pub fn set_minimum(&mut self, v: usize) -> Result<()> {
        catch(|| self.view.setMinimumValue(v as _))
    }

    pub fn maximum(&self) -> Result<usize> {
        catch(|| self.view.maximumValue() as _)
    }

    pub fn set_maximum(&mut self, v: usize) -> Result<()> {
        catch(|| self.view.setMaximumValue(v as _))
    }

    pub fn freq(&self) -> Result<usize> {
        Ok(0)
    }

    pub fn set_freq(&mut self, _v: usize) -> Result<()> {
        Ok(())
    }

    pub fn pos(&self) -> Result<usize> {
        catch(|| self.view.value() as _)
    }

    pub fn set_pos(&mut self, pos: usize) -> Result<()> {
        catch(|| self.view.setValue(pos as _))
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
