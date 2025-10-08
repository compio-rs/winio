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
    GlobalRuntime,
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
    pub fn new(parent: impl AsContainer) -> Self {
        unsafe {
            let parent = parent.as_container();
            let mtm = parent.mtm();

            let view = NSComboBox::new(mtm);
            view.setBezeled(true);
            view.setDrawsBackground(false);
            view.setEditable(false);
            view.setSelectable(false);
            let handle = Widget::from_nsview(&parent, Retained::cast_unchecked(view.clone()));

            let delegate = ComboBoxDelegate::new(mtm);
            let del_obj = ProtocolObject::from_ref(&*delegate);
            view.setDelegate(Some(del_obj));

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

    pub fn text(&self) -> String {
        unsafe { from_nsstring(&self.view.stringValue()) }
    }

    pub fn set_text(&mut self, s: impl AsRef<str>) {
        unsafe {
            self.view.setStringValue(&NSString::from_str(s.as_ref()));
        }
    }

    pub fn selection(&self) -> Option<usize> {
        let index = unsafe { self.view.indexOfSelectedItem() };
        if index < 0 { None } else { Some(index as _) }
    }

    pub fn set_selection(&mut self, i: Option<usize>) {
        unsafe {
            let old_sel = self.selection();
            if i != old_sel {
                if let Some(i) = self.selection() {
                    self.view.deselectItemAtIndex(i as _);
                }
                if let Some(i) = i {
                    self.view.selectItemAtIndex(i as _);
                }
            }
        }
    }

    pub fn is_editable(&self) -> bool {
        unsafe { self.view.isEditable() }
    }

    pub fn set_editable(&mut self, v: bool) {
        unsafe {
            self.view.setDrawsBackground(v);
            self.view.setEditable(v);
            self.view.setSelectable(v);
        }
    }

    pub async fn wait_change(&self) {
        self.delegate.ivars().changed.wait().await
    }

    pub async fn wait_select(&self) {
        self.delegate.ivars().select.wait().await
    }

    pub fn len(&self) -> usize {
        unsafe { self.view.numberOfItems() as _ }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn clear(&mut self) {
        unsafe {
            self.view.removeAllItems();
        }
    }

    pub fn get(&self, i: usize) -> String {
        unsafe {
            let s = Retained::cast_unchecked(self.view.itemObjectValueAtIndex(i as _));
            from_nsstring(&s)
        }
    }

    pub fn set(&mut self, i: usize, s: impl AsRef<str>) {
        self.remove(i);
        self.insert(i, s);
    }

    pub fn insert(&mut self, i: usize, s: impl AsRef<str>) {
        unsafe {
            let s = NSString::from_str(s.as_ref());
            self.view.insertItemWithObjectValue_atIndex(&s, i as _);
        }
        if (!self.is_editable()) && self.len() == 1 {
            self.set_selection(Some(0));
        }
    }

    pub fn remove(&mut self, i: usize) {
        unsafe {
            let i = i as isize;
            let remove_current = self.view.indexOfSelectedItem() == i;
            self.view.removeItemAtIndex(i);
            let len = self.view.numberOfItems();
            if remove_current && (!self.is_editable()) {
                if len > 0 {
                    self.view.selectItemAtIndex(i.min(len - 1));
                } else {
                    self.view.setStringValue(ns_string!(""));
                }
            }
        }
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
