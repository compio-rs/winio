use inherit_methods_macro::inherit_methods;
use objc2::{
    DeclaredClass, MainThreadOnly, define_class, msg_send,
    rc::{Allocated, Retained},
    runtime::ProtocolObject,
};
use objc2_app_kit::{
    NSComboBox, NSComboBoxDelegate, NSControlTextEditingDelegate, NSTextFieldDelegate,
};
use objc2_foundation::{
    MainThreadMarker, NSNotification, NSObject, NSObjectProtocol, NSString, ns_string,
};
use winio_callback::Callback;
use winio_handle::AsContainer;
use winio_primitive::{Point, Size};

use crate::{
    GlobalRuntime, Result, catch,
    ui::{Widget, from_nsstring},
};

#[derive(Debug)]
pub struct ComboBox {
    handle: Widget,
    view: Retained<NSComboBox>,
    delegate: Retained<ComboBoxDelegate>,
}

#[inherit_methods(from = "self.handle")]
impl ComboBox {
    pub fn new(parent: impl AsContainer) -> Result<Self> {
        let parent = parent.as_container();
        let mtm = parent.as_app_kit().mtm();

        catch(|| unsafe {
            let view = NSComboBox::new(mtm);
            view.setBezeled(true);
            view.setDrawsBackground(false);
            view.setEditable(false);
            view.setSelectable(false);
            let handle = Widget::from_nsview(parent, Retained::cast_unchecked(view.clone()))?;

            let delegate = ComboBoxDelegate::new(mtm);
            let del_obj = ProtocolObject::from_ref(&*delegate);
            view.setDelegate(Some(del_obj));

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

    pub fn selection(&self) -> Result<Option<usize>> {
        catch(|| {
            let index = self.view.indexOfSelectedItem();
            if index < 0 { None } else { Some(index as _) }
        })
    }

    pub fn set_selection(&mut self, i: usize) -> Result<()> {
        let old_sel = self.selection()?;
        if Some(i) != old_sel {
            if let Some(i) = self.selection()? {
                catch(|| self.view.deselectItemAtIndex(i as _))?;
            }
            catch(|| self.view.selectItemAtIndex(i as _))?;
        }
        Ok(())
    }

    pub fn is_editable(&self) -> Result<bool> {
        catch(|| self.view.isEditable())
    }

    pub fn set_editable(&mut self, v: bool) -> Result<()> {
        catch(|| {
            self.view.setDrawsBackground(v);
            self.view.setEditable(v);
            self.view.setSelectable(v);
        })
    }

    pub async fn wait_change(&self) {
        self.delegate.ivars().changed.wait().await
    }

    pub async fn wait_select(&self) {
        self.delegate.ivars().select.wait().await
    }

    pub fn len(&self) -> Result<usize> {
        catch(|| self.view.numberOfItems() as _)
    }

    pub fn is_empty(&self) -> Result<bool> {
        Ok(self.len()? == 0)
    }

    pub fn clear(&mut self) -> Result<()> {
        catch(|| self.view.removeAllItems())
    }

    pub fn get(&self, i: usize) -> Result<String> {
        catch(|| unsafe {
            let s = Retained::cast_unchecked(self.view.itemObjectValueAtIndex(i as _));
            from_nsstring(&s)
        })
    }

    pub fn set(&mut self, i: usize, s: impl AsRef<str>) -> Result<()> {
        self.remove(i)?;
        self.insert(i, s)
    }

    pub fn insert(&mut self, i: usize, s: impl AsRef<str>) -> Result<()> {
        catch(|| unsafe {
            let s = NSString::from_str(s.as_ref());
            self.view.insertItemWithObjectValue_atIndex(&s, i as _);
        })?;
        if (!self.is_editable()?) && self.len()? == 1 {
            self.set_selection(0)?;
        }
        Ok(())
    }

    pub fn remove(&mut self, i: usize) -> Result<()> {
        let i = i as isize;
        catch(|| {
            let remove_current = self.view.indexOfSelectedItem() == i;
            self.view.removeItemAtIndex(i);
            let len = self.view.numberOfItems();
            if remove_current && (!self.is_editable()?) {
                if len > 0 {
                    self.view.selectItemAtIndex(i.min(len - 1));
                } else {
                    self.view.setStringValue(ns_string!(""));
                }
            }
            Ok(())
        })
        .flatten()
    }
}

winio_handle::impl_as_widget!(ComboBox, handle);

#[derive(Debug, Default)]
struct ComboBoxDelegateIvars {
    changed: Callback,
    select: Callback,
}

define_class! {
    #[unsafe(super(NSObject))]
    #[name = "WinioComboBoxDelegate"]
    #[ivars = ComboBoxDelegateIvars]
    #[thread_kind = MainThreadOnly]
    #[derive(Debug)]
    struct ComboBoxDelegate;

    #[allow(non_snake_case)]
    impl ComboBoxDelegate {
        #[unsafe(method_id(init))]
        fn init(this: Allocated<Self>) -> Option<Retained<Self>> {
            let this = this.set_ivars(ComboBoxDelegateIvars::default());
            unsafe { msg_send![super(this), init] }
        }
    }

    unsafe impl NSObjectProtocol for ComboBoxDelegate {}

    #[allow(non_snake_case)]
    unsafe impl NSControlTextEditingDelegate for ComboBoxDelegate {
        #[unsafe(method(controlTextDidChange:))]
        fn controlTextDidChange(&self, _notification: &NSNotification) {
            self.ivars().changed.signal::<GlobalRuntime>(());
        }
    }

    unsafe impl NSTextFieldDelegate for ComboBoxDelegate {}

    #[allow(non_snake_case)]
    unsafe impl NSComboBoxDelegate for ComboBoxDelegate {
        #[unsafe(method(comboBoxSelectionDidChange:))]
        unsafe fn comboBoxSelectionDidChange(&self, _notification: &NSNotification) {
            self.ivars().select.signal::<GlobalRuntime>(());
        }
    }
}

impl ComboBoxDelegate {
    pub fn new(mtm: MainThreadMarker) -> Retained<Self> {
        unsafe { msg_send![mtm.alloc::<Self>(), init] }
    }
}
