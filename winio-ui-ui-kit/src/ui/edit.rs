use inherit_methods_macro::inherit_methods;
use objc2::{
    DeclaredClass, MainThreadOnly, define_class, msg_send,
    rc::{Allocated, Retained},
    runtime::ProtocolObject,
    sel,
};
use objc2_foundation::{MainThreadMarker, NSObject, NSObjectProtocol, NSString};
use objc2_ui_kit::{
    NSTextAlignment, UIControlEvents, UITextBorderStyle, UITextField, UITextFieldDelegate,
    UITextInputTraits,
};
use winio_callback::Callback;
use winio_handle::AsContainer;
use winio_primitive::{HAlign, Point, Size};

use crate::{GlobalRuntime, Result, catch, ui::Widget};

#[derive(Debug)]
pub struct Edit {
    handle: Widget,
    view: Retained<UITextField>,
    delegate: Retained<EditDelegate>,
    password: bool,
}

#[inherit_methods(from = "self.handle")]
impl Edit {
    pub fn new(parent: impl AsContainer) -> Result<Self> {
        let parent = parent.as_container();
        let mtm = parent.as_ui_kit().mtm();

        catch(|| unsafe {
            let view = UITextField::new(mtm);
            view.setBorderStyle(UITextBorderStyle::RoundedRect);

            let delegate = EditDelegate::new(mtm);
            let del_obj = ProtocolObject::from_ref(&*delegate);
            view.setDelegate(Some(del_obj));

            view.addTarget_action_forControlEvents(
                Some(&delegate),
                sel!(onChange),
                UIControlEvents::EditingChanged,
            );

            let handle = Widget::from_uiview(parent, Retained::cast_unchecked(view.clone()))?;

            Ok(Self {
                handle,
                view,
                delegate,
                password: false,
            })
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

    pub fn text(&self) -> Result<String> {
        catch(|| crate::from_nsstring(&self.view.text().unwrap_or_default()))
    }

    pub fn set_text(&mut self, s: impl AsRef<str>) -> Result<()> {
        let ns = NSString::from_str(s.as_ref());
        catch(|| self.view.setText(Some(&ns)))
    }

    pub fn halign(&self) -> Result<HAlign> {
        catch(|| {
            let raw = self.view.textAlignment();
            match raw {
                NSTextAlignment::Center => HAlign::Center,
                NSTextAlignment::Right => HAlign::Right,
                NSTextAlignment::Justified => HAlign::Stretch,
                _ => HAlign::Left,
            }
        })
    }

    pub fn set_halign(&mut self, align: HAlign) -> Result<()> {
        let raw = match align {
            HAlign::Left => NSTextAlignment::Left,
            HAlign::Center => NSTextAlignment::Center,
            HAlign::Right => NSTextAlignment::Right,
            HAlign::Stretch => NSTextAlignment::Justified,
        };
        catch(|| self.view.setTextAlignment(raw))
    }

    pub fn is_password(&self) -> Result<bool> {
        Ok(self.password)
    }

    pub fn set_password(&mut self, v: bool) -> Result<()> {
        catch(|| self.view.setSecureTextEntry(v))?;
        self.password = v;
        Ok(())
    }

    pub fn is_readonly(&self) -> Result<bool> {
        catch(|| !self.view.isEnabled())
    }

    pub fn set_readonly(&mut self, v: bool) -> Result<()> {
        catch(|| self.view.setEnabled(!v))
    }

    pub async fn wait_change(&self) {
        self.delegate.ivars().changed.wait().await
    }
}

winio_handle::impl_as_widget!(Edit, handle);

#[derive(Debug, Default)]
struct EditDelegateIvars {
    changed: Callback,
}

define_class! {
    #[unsafe(super(NSObject))]
    #[name = "WinioEditDelegateUIKit"]
    #[ivars = EditDelegateIvars]
    #[thread_kind = MainThreadOnly]
    #[derive(Debug)]
    struct EditDelegate;

    #[allow(non_snake_case)]
    impl EditDelegate {
        #[unsafe(method_id(init))]
        fn init(this: Allocated<Self>) -> Option<Retained<Self>> {
            let this = this.set_ivars(EditDelegateIvars::default());
            unsafe { msg_send![super(this), init] }
        }

        #[unsafe(method(onChange))]
        unsafe fn onChange(&self) {
            self.ivars().changed.signal::<GlobalRuntime>(());
        }
    }

    unsafe impl NSObjectProtocol for EditDelegate {}

    unsafe impl UITextFieldDelegate for EditDelegate {}
}

impl EditDelegate {
    pub fn new(mtm: MainThreadMarker) -> Retained<Self> {
        unsafe { msg_send![mtm.alloc::<Self>(), init] }
    }
}
