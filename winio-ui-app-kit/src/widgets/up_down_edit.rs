use inherit_methods_macro::inherit_methods;
use objc2::{
    DeclaredClass, MainThreadOnly, define_class, msg_send, rc::Retained, runtime::ProtocolObject,
    sel,
};
use objc2_app_kit::{
    NSControlTextEditingDelegate, NSStepper, NSTextField, NSTextFieldDelegate, NSView,
};
use objc2_foundation::{
    MainThreadMarker, NSNotification, NSObject, NSObjectProtocol, NSPoint, NSRect, NSSize, NSString,
};
use winio_callback::Callback;
use winio_handle::AsContainer;
use winio_primitive::{Point, Size};

use crate::{GlobalRuntime, Result, catch, from_nsstring, widgets::Widget};

#[derive(Debug)]
pub struct UpDownEdit {
    handle: Widget,
    text: Retained<NSTextField>,
    stepper: Retained<NSStepper>,
    delegate: Retained<UpDownEditDelegate>,
}

#[inherit_methods(from = "self.handle")]
impl UpDownEdit {
    pub fn new(parent: impl AsContainer) -> Result<Self> {
        let parent = parent.as_container();
        let mtm = parent.as_app_kit().mtm();

        catch(|| unsafe {
            let container = NSView::new(mtm);
            let text = NSTextField::new(mtm);
            text.setBezeled(true);
            text.setDrawsBackground(true);
            text.setEditable(true);
            text.setSelectable(true);
            text.setStringValue(&NSString::from_str("0"));

            let stepper = NSStepper::new(mtm);
            stepper.setValueWraps(false);
            stepper.setAutorepeat(true);

            let delegate = UpDownEditDelegate::new(mtm, text.clone(), stepper.clone());
            let del_obj = ProtocolObject::from_ref(&*delegate);
            text.setDelegate(Some(del_obj));
            stepper.setTarget(Some(&delegate));
            stepper.setAction(Some(sel!(onStep)));

            container.addSubview(&text);
            container.addSubview(&stepper);

            let handle = Widget::from_nsview(parent, container)?;

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
        })
    }

    pub fn preferred_size(&self) -> Result<Size> {
        let text = catch(|| self.text.fittingSize())?;
        let stepper = catch(|| self.stepper.fittingSize())?;
        let width = text.width.max(60.0) + stepper.width.max(19.0);
        let height = text.height.max(stepper.height).max(24.0);
        Ok(Size::new(width, height))
    }

    pub fn loc(&self) -> Result<Point>;

    pub fn set_loc(&mut self, p: Point) -> Result<()>;

    pub fn size(&self) -> Result<Size>;

    pub fn set_size(&mut self, v: Size) -> Result<()> {
        self.handle.set_size(v)?;
        let stepper_width = catch(|| self.stepper.fittingSize().width)?.max(19.0);
        let text_width = (v.width - stepper_width).max(0.0);
        catch(|| {
            self.text.setFrame(NSRect::new(
                NSPoint::ZERO,
                NSSize::new(text_width, v.height),
            ));
            self.stepper.setFrame(NSRect::new(
                NSPoint::new(text_width, 0.0),
                NSSize::new(stepper_width, v.height),
            ));
        })
    }

    pub fn tooltip(&self) -> Result<String>;

    pub fn set_tooltip(&mut self, s: impl AsRef<str>) -> Result<()> {
        self.handle.set_tooltip(&s)?;
        let s = NSString::from_str(s.as_ref());
        catch(|| {
            self.text.setToolTip(Some(&s));
            self.stepper.setToolTip(Some(&s));
        })
    }

    pub fn minimum(&self) -> Result<usize> {
        catch(|| self.stepper.minValue() as _)
    }

    pub fn set_minimum(&mut self, v: usize) -> Result<()> {
        catch(|| self.stepper.setMinValue(v as _))
    }

    pub fn maximum(&self) -> Result<usize> {
        catch(|| self.stepper.maxValue() as _)
    }

    pub fn set_maximum(&mut self, v: usize) -> Result<()> {
        catch(|| self.stepper.setMaxValue(v as _))
    }

    pub fn step(&self) -> Result<usize> {
        catch(|| self.stepper.increment() as _)
    }

    pub fn set_step(&mut self, v: usize) -> Result<()> {
        catch(|| self.stepper.setIncrement(v as _))
    }

    pub fn value(&self) -> Result<usize> {
        catch(|| self.stepper.doubleValue() as _)
    }

    pub fn set_value(&mut self, v: usize) -> Result<()> {
        catch(|| {
            self.stepper.setDoubleValue(v as _);
            self.text
                .setStringValue(&NSString::from_str(&v.to_string()));
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
    text: Retained<NSTextField>,
    stepper: Retained<NSStepper>,
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
            let value = self.ivars().stepper.doubleValue();
            self.ivars()
                .text
                .setStringValue(&NSString::from_str(&(value as usize).to_string()));
            self.ivars().action.signal::<GlobalRuntime>(());
        }
    }

    unsafe impl NSObjectProtocol for UpDownEditDelegate {}

    #[allow(non_snake_case)]
    unsafe impl NSControlTextEditingDelegate for UpDownEditDelegate {
        #[unsafe(method(controlTextDidEndEditing:))]
        fn controlTextDidEndEditing(&self, _notification: &NSNotification) {
            self.commit();
        }
    }

    unsafe impl NSTextFieldDelegate for UpDownEditDelegate {}
}

impl UpDownEditDelegate {
    fn new(
        mtm: MainThreadMarker,
        text: Retained<NSTextField>,
        stepper: Retained<NSStepper>,
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
        let text = from_nsstring(&self.ivars().text.stringValue());
        let stepper = &self.ivars().stepper;
        let min = stepper.minValue();
        let max = stepper.maxValue();
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
            Err(_) => stepper.doubleValue(),
        };
        stepper.setDoubleValue(value);
        self.ivars()
            .text
            .setStringValue(&NSString::from_str(&(value as i64).to_string()));
        self.ivars().action.signal::<GlobalRuntime>(());
    }
}
