use objc2::{
    ClassType, DeclaredClass, declare_class, msg_send_id,
    mutability::MainThreadOnly,
    rc::{Allocated, Id},
    runtime::ProtocolObject,
};
use objc2_app_kit::{
    NSComboBox, NSComboBoxDelegate, NSControlTextEditingDelegate, NSTextFieldDelegate,
};
use objc2_foundation::{MainThreadMarker, NSNotification, NSObject, NSObjectProtocol, NSString};

use crate::{
    AsRawWindow, AsWindow, Point, Size,
    ui::{Callback, Widget, from_nsstring},
};

#[derive(Debug)]
pub struct ComboBoxImpl<const E: bool> {
    handle: Widget,
    view: Id<NSComboBox>,
    delegate: Id<ComboBoxDelegate>,
}

impl<const E: bool> ComboBoxImpl<E> {
    pub fn new(parent: impl AsWindow) -> Self {
        unsafe {
            let mtm = MainThreadMarker::new().unwrap();

            let view = NSComboBox::new(mtm);
            view.setBezeled(true);
            view.setDrawsBackground(E);
            view.setEditable(E);
            view.setSelectable(E);
            let handle =
                Widget::from_nsview(parent.as_window().as_raw_window(), Id::cast(view.clone()));

            let delegate = ComboBoxDelegate::new(mtm);
            let del_obj = ProtocolObject::from_id(delegate.clone());
            view.setDelegate(Some(&del_obj));

            Self {
                handle,
                view,
                delegate,
            }
        }
    }

    pub fn preferred_size(&self) -> Size {
        self.handle.preferred_size()
    }

    pub fn loc(&self) -> Point {
        self.handle.loc()
    }

    pub fn set_loc(&mut self, p: Point) {
        self.handle.set_loc(p)
    }

    pub fn size(&self) -> Size {
        self.handle.size()
    }

    pub fn set_size(&mut self, v: Size) {
        self.handle.set_size(v)
    }

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
            if let Some(i) = i {
                self.view.selectItemAtIndex(i as _);
            } else if let Some(i) = self.selection() {
                self.view.deselectItemAtIndex(i as _);
            }
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

    pub fn clear(&mut self) {
        unsafe {
            self.view.removeAllItems();
        }
    }

    pub fn get(&self, i: usize) -> String {
        unsafe {
            let s = Id::cast(self.view.itemObjectValueAtIndex(i as _));
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
    }

    pub fn remove(&mut self, i: usize) {
        unsafe {
            self.view.removeItemAtIndex(i as _);
        }
    }
}

pub type ComboBox = ComboBoxImpl<false>;
pub type ComboEntry = ComboBoxImpl<true>;

#[derive(Default, Clone)]
struct ComboBoxDelegateIvars {
    changed: Callback,
    select: Callback,
}

declare_class! {
    #[derive(Debug)]
    struct ComboBoxDelegate;

    unsafe impl ClassType for ComboBoxDelegate {
        type Super = NSObject;
        type Mutability = MainThreadOnly;
        const NAME: &'static str = "WinioComboBoxDelegate";
    }

    impl DeclaredClass for ComboBoxDelegate {
        type Ivars = ComboBoxDelegateIvars;
    }

    #[allow(non_snake_case)]
    unsafe impl ComboBoxDelegate {
        #[method_id(init)]
        fn init(this: Allocated<Self>) -> Option<Id<Self>> {
            let this = this.set_ivars(ComboBoxDelegateIvars::default());
            unsafe { msg_send_id![super(this), init] }
        }
    }

    unsafe impl NSObjectProtocol for ComboBoxDelegate {}

    #[allow(non_snake_case)]
    unsafe impl NSControlTextEditingDelegate for ComboBoxDelegate {
        #[method(controlTextDidChange:)]
        fn controlTextDidChange(&self, _notification: &NSNotification) {
            self.ivars().changed.signal(());
        }
    }

    unsafe impl NSTextFieldDelegate for ComboBoxDelegate {}

    #[allow(non_snake_case)]
    unsafe impl NSComboBoxDelegate for ComboBoxDelegate {
        #[method(comboBoxSelectionDidChange:)]
        unsafe fn comboBoxSelectionDidChange(&self, _notification: &NSNotification) {
            self.ivars().select.signal(());
        }
    }
}

impl ComboBoxDelegate {
    pub fn new(mtm: MainThreadMarker) -> Id<Self> {
        unsafe { msg_send_id![mtm.alloc::<Self>(), init] }
    }
}