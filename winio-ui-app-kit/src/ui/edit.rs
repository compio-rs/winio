use inherit_methods_macro::inherit_methods;
use objc2::{
    DeclaredClass, MainThreadOnly, define_class, msg_send,
    rc::{Allocated, Retained},
    runtime::ProtocolObject,
};
use objc2_app_kit::{
    NSControlTextEditingDelegate, NSSecureTextField, NSTextAlignment, NSTextField,
    NSTextFieldDelegate,
};
use objc2_foundation::{MainThreadMarker, NSNotification, NSObject, NSObjectProtocol};
use winio_callback::Callback;
use winio_handle::{AsContainer, BorrowedContainer};
use winio_primitive::{HAlign, Point, Size};

use crate::{GlobalRuntime, Result, catch, ui::Widget};

#[derive(Debug)]
struct EditImpl {
    handle: Widget,
    view: Retained<NSTextField>,
    delegate: Retained<EditDelegate>,
}

#[inherit_methods(from = "self.handle")]
impl EditImpl {
    pub fn new(
        parent: impl AsContainer,
        view: Retained<NSTextField>,
        delegate: Retained<EditDelegate>,
    ) -> Result<Self> {
        catch(|| unsafe {
            view.setBezeled(true);
            view.setDrawsBackground(true);
            view.setEditable(true);
            view.setSelectable(true);
            let del_obj = ProtocolObject::from_ref(&*delegate);
            view.setDelegate(Some(del_obj));

            let handle = Widget::from_nsview(parent, Retained::cast_unchecked(view.clone()))?;

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

    pub fn preferred_size(&self) -> Result<Size>;

    pub fn loc(&self) -> Result<Point>;

    pub fn set_loc(&mut self, p: Point) -> Result<()>;

    pub fn size(&self) -> Result<Size>;

    pub fn set_size(&mut self, v: Size) -> Result<()>;

    pub fn tooltip(&self) -> Result<String>;

    pub fn set_tooltip(&mut self, s: impl AsRef<str>) -> Result<()>;

    pub fn text(&self) -> Result<String>;

    pub fn set_text(&mut self, s: impl AsRef<str>) -> Result<()>;

    pub fn halign(&self) -> Result<HAlign> {
        let align = catch(|| self.view.alignment())?;
        let align = match align {
            NSTextAlignment::Right => HAlign::Right,
            NSTextAlignment::Center => HAlign::Center,
            NSTextAlignment::Justified => HAlign::Stretch,
            _ => HAlign::Left,
        };
        Ok(align)
    }

    pub fn set_halign(&mut self, align: HAlign) -> Result<()> {
        let align = match align {
            HAlign::Left => NSTextAlignment::Left,
            HAlign::Center => NSTextAlignment::Center,
            HAlign::Right => NSTextAlignment::Right,
            HAlign::Stretch => NSTextAlignment::Justified,
        };
        catch(|| self.view.setAlignment(align))
    }

    pub fn is_readonly(&self) -> Result<bool> {
        catch(|| !self.view.isEditable())
    }

    pub fn set_readonly(&mut self, v: bool) -> Result<()> {
        catch(|| self.view.setEditable(!v))
    }

    pub async fn wait_change(&self) {
        self.delegate.ivars().changed.wait().await
    }
}

winio_handle::impl_as_widget!(EditImpl, handle);

#[derive(Debug)]
pub struct Edit {
    handle: EditImpl,
    password: bool,
}

#[inherit_methods(from = "self.handle")]
impl Edit {
    pub fn new(parent: impl AsContainer) -> Result<Self> {
        let parent = parent.as_container();
        let mtm = parent.as_app_kit().mtm();

        catch(|| {
            let view = NSTextField::new(mtm);
            let delegate = EditDelegate::new(mtm);
            let handle = EditImpl::new(parent, view, delegate)?;
            Ok(Self {
                handle,
                password: false,
            })
        })
        .flatten()
    }

    fn recreate(&mut self, password: bool, mtm: MainThreadMarker) -> Result<()> {
        let view = catch(|| unsafe {
            if password {
                Retained::cast_unchecked(NSSecureTextField::new(mtm))
            } else {
                NSTextField::new(mtm)
            }
        })?;
        let parent = self.handle.handle.parent()?;
        let mut new_handle = EditImpl::new(
            BorrowedContainer::app_kit(&parent),
            view,
            self.handle.delegate.clone(),
        )?;
        new_handle.set_visible(self.handle.is_visible()?)?;
        new_handle.set_enabled(self.handle.is_enabled()?)?;
        new_handle.set_loc(self.handle.loc()?)?;
        new_handle.set_size(self.handle.size()?)?;
        new_handle.set_text(self.handle.text()?)?;
        new_handle.set_halign(self.handle.halign()?)?;
        self.handle = new_handle;
        Ok(())
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

    pub fn text(&self) -> Result<String>;

    pub fn set_text(&mut self, s: impl AsRef<str>) -> Result<()>;

    pub fn is_password(&self) -> Result<bool> {
        Ok(self.password)
    }

    pub fn set_password(&mut self, v: bool) -> Result<()> {
        if self.password != v {
            self.recreate(v, self.handle.view.mtm())?;
            self.password = v;
        }
        Ok(())
    }

    pub fn halign(&self) -> Result<HAlign>;

    pub fn set_halign(&mut self, align: HAlign) -> Result<()>;

    pub fn is_readonly(&self) -> Result<bool> {
        if self.is_password()? {
            Ok(false)
        } else {
            self.handle.is_readonly()
        }
    }

    pub fn set_readonly(&mut self, v: bool) -> Result<()> {
        if !self.is_password()? {
            self.handle.set_readonly(v)?;
        }
        Ok(())
    }

    pub async fn wait_change(&self) {
        self.handle.wait_change().await
    }
}

winio_handle::impl_as_widget!(Edit, handle);

#[derive(Debug, Default)]
struct EditDelegateIvars {
    changed: Callback,
}

define_class! {
    #[unsafe(super(NSObject))]
    #[name = "WinioEditDelegate"]
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
    }

    unsafe impl NSObjectProtocol for EditDelegate {}

    #[allow(non_snake_case)]
    unsafe impl NSControlTextEditingDelegate for EditDelegate {
        #[unsafe(method(controlTextDidChange:))]
        fn controlTextDidChange(&self, _notification: &NSNotification) {
            self.ivars().changed.signal::<GlobalRuntime>(());
        }
    }

    unsafe impl NSTextFieldDelegate for EditDelegate {}
}

impl EditDelegate {
    pub fn new(mtm: MainThreadMarker) -> Retained<Self> {
        unsafe { msg_send![mtm.alloc::<Self>(), init] }
    }
}
