use inherit_methods_macro::inherit_methods;
use objc2::{
    DeclaredClass, MainThreadOnly, define_class, msg_send,
    rc::{Allocated, Retained},
    runtime::ProtocolObject,
};
use objc2_foundation::{MainThreadMarker, NSObject, NSObjectProtocol, NSString};
use objc2_ui_kit::{
    UIPickerView, UIPickerViewDataSource, UIPickerViewDelegate, UITextBorderStyle, UITextField,
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
    view: Retained<UITextField>,
    picker: Retained<UIPickerView>,
    delegate: Retained<ComboBoxDelegate>,
    items: std::cell::RefCell<Vec<String>>,
    editable: bool,
}

#[inherit_methods(from = "self.handle")]
impl ComboBox {
    pub fn new(parent: impl AsContainer) -> Result<Self> {
        let parent = parent.as_container();
        let mtm = parent.as_ui_kit().mtm();

        catch(|| unsafe {
            let view = UITextField::new(mtm);
            view.setBorderStyle(UITextBorderStyle::RoundedRect);
            view.setEnabled(false);

            let picker = UIPickerView::new(mtm);

            let delegate = ComboBoxDelegate::new(mtm);
            let del_obj = ProtocolObject::from_ref(&*delegate);
            picker.setDelegate(Some(del_obj));
            let del_obj = ProtocolObject::from_ref(&*delegate);
            picker.setDataSource(Some(del_obj));

            view.setInputView(Some(&picker));

            let handle = Widget::from_uiview(parent, Retained::cast_unchecked(view.clone()))?;

            Ok(Self {
                handle,
                view,
                picker,
                delegate,
                items: std::cell::RefCell::new(Vec::new()),
                editable: false,
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
        catch(|| from_nsstring(&self.view.text().unwrap_or_default()))
    }

    pub fn set_text(&mut self, s: impl AsRef<str>) -> Result<()> {
        let ns = NSString::from_str(s.as_ref());
        catch(|| self.view.setText(Some(&ns)))
    }

    pub fn selection(&self) -> Result<Option<usize>> {
        catch(|| {
            let index = self.picker.selectedRowInComponent(0);
            let len = self.items.borrow().len();
            if index < 0 || len == 0 || (index as usize) >= len {
                None
            } else {
                Some(index as _)
            }
        })
    }

    pub fn set_selection(&mut self, i: usize) -> Result<()> {
        catch(|| {
            self.picker
                .selectRow_inComponent_animated(i as isize, 0, false);
            let items = self.items.borrow();
            if let Some(text) = items.get(i) {
                self.view.setText(Some(&NSString::from_str(text)));
            }
        })
    }

    pub fn is_editable(&self) -> Result<bool> {
        Ok(self.editable)
    }

    pub fn set_editable(&mut self, v: bool) -> Result<()> {
        catch(|| self.view.setEnabled(v))?;
        self.editable = v;
        Ok(())
    }

    pub async fn wait_change(&self) {
        self.delegate.ivars().changed.wait().await
    }

    pub async fn wait_select(&self) {
        self.delegate.ivars().select.wait().await
    }

    pub fn len(&self) -> Result<usize> {
        Ok(self.items.borrow().len())
    }

    pub fn is_empty(&self) -> Result<bool> {
        Ok(self.len()? == 0)
    }

    pub fn clear(&mut self) -> Result<()> {
        self.items.borrow_mut().clear();
        catch(|| self.picker.reloadAllComponents())
    }

    pub fn get(&self, i: usize) -> Result<String> {
        Ok(self.items.borrow()[i].clone())
    }

    pub fn set(&mut self, i: usize, s: impl AsRef<str>) -> Result<()> {
        self.items.borrow_mut()[i] = s.as_ref().to_string();
        catch(|| self.picker.reloadAllComponents())
    }

    pub fn insert(&mut self, i: usize, s: impl AsRef<str>) -> Result<()> {
        self.items.borrow_mut().insert(i, s.as_ref().to_string());
        catch(|| self.picker.reloadAllComponents())?;
        if !self.is_editable()? && self.len()? == 1 {
            self.set_selection(0)?;
        }
        Ok(())
    }

    pub fn remove(&mut self, i: usize) -> Result<()> {
        self.items.borrow_mut().remove(i);
        catch(|| self.picker.reloadAllComponents())
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
    #[name = "WinioComboBoxDelegateUIKit"]
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
    unsafe impl UIPickerViewDelegate for ComboBoxDelegate {
        #[unsafe(method(pickerView:didSelectRow:inComponent:))]
        unsafe fn pickerView_didSelectRow_inComponent(
            &self,
            _picker_view: &UIPickerView,
            _row: isize,
            _component: isize,
        ) {
            self.ivars().select.signal::<GlobalRuntime>(());
        }
    }

    #[allow(non_snake_case)]
    unsafe impl UIPickerViewDataSource for ComboBoxDelegate {
        #[unsafe(method(numberOfComponentsInPickerView:))]
        unsafe fn numberOfComponentsInPickerView(&self, _picker_view: &UIPickerView) -> isize {
            1
        }

        #[unsafe(method(pickerView:numberOfRowsInComponent:))]
        unsafe fn pickerView_numberOfRowsInComponent(
            &self,
            _picker_view: &UIPickerView,
            _component: isize,
        ) -> isize {
            0
        }
    }
}

impl ComboBoxDelegate {
    pub fn new(mtm: MainThreadMarker) -> Retained<Self> {
        unsafe { msg_send![mtm.alloc::<Self>(), init] }
    }
}
