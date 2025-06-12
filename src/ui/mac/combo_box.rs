use objc2::{
    DeclaredClass, MainThreadOnly, define_class, msg_send,
    rc::{Allocated, Retained},
    runtime::ProtocolObject,
};
use objc2_app_kit::{
    NSComboBox, NSComboBoxDelegate, NSControlTextEditingDelegate, NSTextFieldDelegate,
};
use objc2_foundation::{MainThreadMarker, NSNotification, NSObject, NSObjectProtocol, NSString};

use crate::{
    AsWindow, Point, Size,
    ui::{Callback, Widget, from_nsstring},
};

#[derive(Debug)]
pub struct ComboBoxImpl<const E: bool> {
    handle: Widget,
    view: Retained<NSComboBox>,
    delegate: Retained<ComboBoxDelegate>,
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
            let handle = Widget::from_nsview(parent, Retained::cast_unchecked(view.clone()));

            let delegate = ComboBoxDelegate::new(mtm);
            let del_obj = ProtocolObject::from_retained(delegate.clone());
            view.setDelegate(Some(&del_obj));

            Self {
                handle,
                view,
                delegate,
            }
        }
    }

    pub fn is_visible(&self) -> bool {
        self.handle.is_visible()
    }

    pub fn set_visible(&mut self, v: bool) {
        self.handle.set_visible(v);
    }

    pub fn preferred_size(&self) -> Size {
        let old_selection = self.selection();
        let mut size = self.handle.preferred_size();
        for i in 0..self.len() {
            self.set_selection_impl(Some(i));
            let new_size = self.handle.preferred_size();
            size.width = size.width.max(new_size.width);
            size.height = size.height.max(new_size.height);
        }
        self.set_selection_impl(old_selection);
        size
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

    fn set_selection_impl(&self, i: Option<usize>) {
        unsafe {
            if let Some(i) = self.selection() {
                self.view.deselectItemAtIndex(i as _);
            }
            if let Some(i) = i {
                self.view.selectItemAtIndex(i as _);
            }
        }
    }

    pub fn set_selection(&mut self, i: Option<usize>) {
        self.set_selection_impl(i);
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
    }

    pub fn remove(&mut self, i: usize) {
        unsafe {
            self.view.removeItemAtIndex(i as _);
        }
    }
}

pub type ComboBox = ComboBoxImpl<false>;
pub type ComboEntry = ComboBoxImpl<true>;

#[derive(Debug, Default, Clone)]
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
            self.ivars().changed.signal(());
        }
    }

    unsafe impl NSTextFieldDelegate for ComboBoxDelegate {}

    #[allow(non_snake_case)]
    unsafe impl NSComboBoxDelegate for ComboBoxDelegate {
        #[unsafe(method(comboBoxSelectionDidChange:))]
        unsafe fn comboBoxSelectionDidChange(&self, _notification: &NSNotification) {
            self.ivars().select.signal(());
        }
    }
}

impl ComboBoxDelegate {
    pub fn new(mtm: MainThreadMarker) -> Retained<Self> {
        unsafe { msg_send![mtm.alloc::<Self>(), init] }
    }
}
