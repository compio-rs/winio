use inherit_methods_macro::inherit_methods;
use objc2::{
    DeclaredClass, MainThreadOnly, define_class, msg_send,
    rc::Retained,
    runtime::{AnyObject, ProtocolObject},
    sel,
};
use objc2_core_foundation::{CGPoint, CGRect, CGSize};
use objc2_foundation::{MainThreadMarker, NSObject, NSObjectProtocol, NSString};
use objc2_ui_kit::{
    UIControlEvents, UIKeyboardType, UIStepper, UITextBorderStyle, UITextField,
    UITextFieldDelegate, UITextInputTraits, UIView,
};
use winio_callback::Callback;
use winio_handle::AsContainer;
use winio_primitive::{Point, Size};

use crate::{GlobalRuntime, Result, catch, from_nsstring, widgets::Widget};

#[derive(Debug)]
pub struct UpDownEdit {
    handle: Widget,
    text: Retained<UITextField>,
    stepper: Retained<UIStepper>,
    delegate: Retained<UpDownEditDelegate>,
}

#[inherit_methods(from = "self.handle")]
impl UpDownEdit {
    pub fn new(parent: impl AsContainer) -> Result<Self> {
        let parent = parent.as_container();
        let mtm = parent.as_ui_kit().mtm();

        catch(|| unsafe {
            let container = UIView::new(mtm);
            let text = UITextField::new(mtm);
            text.setBorderStyle(UITextBorderStyle::RoundedRect);
            text.setKeyboardType(UIKeyboardType::NumberPad);
            text.setText(Some(&NSString::from_str("0")));

            let stepper = UIStepper::new(mtm);

            let delegate = UpDownEditDelegate::new(mtm, text.clone(), stepper.clone());
            text.setDelegate(Some(ProtocolObject::from_ref(&*delegate)));
            let obj: &AnyObject = &delegate;
            stepper.addTarget_action_forControlEvents(
                Some(obj),
                sel!(onStep),
                UIControlEvents::ValueChanged,
            );

            container.addSubview(&text);
            container.addSubview(&stepper);

            let handle = Widget::from_uiview(parent, container)?;

            Ok(Self {
                handle,
                text,
                stepper,
                delegate,
            })
        })
        .flatten()
    }

    pub fn is_visible(&self) -> Result<bool>;

    pub fn set_visible(&mut self, v: bool) -> Result<()>;

    pub fn is_enabled(&self) -> Result<bool> {
        Ok(catch(|| self.text.isEnabled())? && catch(|| self.stepper.isEnabled())?)
    }

    pub fn set_enabled(&mut self, v: bool) -> Result<()> {
        catch(|| {
            self.text.setEnabled(v);
            self.stepper.setEnabled(v);
        })?;
        self.handle.set_enabled(v)
    }

    pub fn preferred_size(&self) -> Result<Size> {
        let text = catch(|| self.text.sizeThatFits(CGSize::new(1000.0, 1000.0)))?;
        Ok(Size::new(
            text.width.max(60.0) + 94.0,
            text.height.max(34.0),
        ))
    }

    pub fn loc(&self) -> Result<Point>;

    pub fn set_loc(&mut self, p: Point) -> Result<()>;

    pub fn size(&self) -> Result<Size>;

    pub fn set_size(&mut self, v: Size) -> Result<()> {
        self.handle.set_size(v)?;
        let stepper_width = 94.0_f64.min(v.width);
        let text_width = (v.width - stepper_width).max(0.0);
        catch(|| {
            self.text.setFrame(CGRect::new(
                CGPoint::ZERO,
                CGSize::new(text_width, v.height),
            ));
            self.stepper.setFrame(CGRect::new(
                CGPoint::new(text_width, 0.0),
                CGSize::new(stepper_width, v.height),
            ));
        })
    }

    pub fn tooltip(&self) -> Result<String>;

    pub fn set_tooltip(&mut self, s: impl AsRef<str>) -> Result<()>;

    pub fn minimum(&self) -> Result<usize> {
        catch(|| self.stepper.minimumValue() as _)
    }

    pub fn set_minimum(&mut self, v: usize) -> Result<()> {
        catch(|| self.stepper.setMinimumValue(v as _))
    }

    pub fn maximum(&self) -> Result<usize> {
        catch(|| self.stepper.maximumValue() as _)
    }

    pub fn set_maximum(&mut self, v: usize) -> Result<()> {
        catch(|| self.stepper.setMaximumValue(v as _))
    }

    pub fn step(&self) -> Result<usize> {
        catch(|| self.stepper.stepValue() as _)
    }

    pub fn set_step(&mut self, v: usize) -> Result<()> {
        catch(|| self.stepper.setStepValue(v as _))
    }

    pub fn value(&self) -> Result<usize> {
        catch(|| self.stepper.value() as _)
    }

    pub fn set_value(&mut self, v: usize) -> Result<()> {
        catch(|| {
            self.stepper.setValue(v as _);
            self.text.setText(Some(&NSString::from_str(&v.to_string())));
        })
    }

    pub async fn wait_change(&self) {
        self.delegate.ivars().action.wait().await
    }
}

winio_handle::impl_as_widget!(UpDownEdit, handle);

#[derive(Debug)]
struct UpDownEditDelegateIvars {
    action: Callback,
    text: Retained<UITextField>,
    stepper: Retained<UIStepper>,
}

define_class! {
    #[unsafe(super(NSObject))]
    #[name = "WinioUpDownEditDelegate"]
    #[ivars = UpDownEditDelegateIvars]
    #[thread_kind = MainThreadOnly]
    #[derive(Debug)]
    struct UpDownEditDelegate;

    #[allow(non_snake_case)]
    impl UpDownEditDelegate {
        #[unsafe(method(onStep))]
        unsafe fn onStep(&self) {
            let value = self.ivars().stepper.value();
            self.ivars()
                .text
                .setText(Some(&NSString::from_str(&(value as usize).to_string())));
            self.ivars().action.signal::<GlobalRuntime>(());
        }
    }

    unsafe impl NSObjectProtocol for UpDownEditDelegate {}

    #[allow(non_snake_case)]
    unsafe impl UITextFieldDelegate for UpDownEditDelegate {
        #[unsafe(method(textFieldDidEndEditing:))]
        fn textFieldDidEndEditing(&self, _text_field: &UITextField) {
            self.commit();
        }
    }
}

impl UpDownEditDelegate {
    fn new(
        mtm: MainThreadMarker,
        text: Retained<UITextField>,
        stepper: Retained<UIStepper>,
    ) -> Retained<Self> {
        let this = mtm.alloc::<Self>();
        let this = this.set_ivars(UpDownEditDelegateIvars {
            action: Callback::new(),
            text,
            stepper,
        });
        unsafe { msg_send![super(this), init] }
    }

    fn commit(&self) {
        let text = self
            .ivars()
            .text
            .text()
            .map(|text| from_nsstring(&text))
            .unwrap_or_default();
        let stepper = &self.ivars().stepper;
        let min = stepper.minimumValue();
        let max = stepper.maximumValue();
        let value = match text.trim().parse::<i64>() {
            Ok(v) => {
                let v = v as f64;
                if v < min {
                    min
                } else if v > max {
                    max
                } else {
                    v
                }
            }
            Err(_) => stepper.value(),
        };
        stepper.setValue(value);
        self.ivars()
            .text
            .setText(Some(&NSString::from_str(&(value as i64).to_string())));
        self.ivars().action.signal::<GlobalRuntime>(());
    }
}
