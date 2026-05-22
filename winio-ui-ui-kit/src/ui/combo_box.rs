use std::cell::RefCell;

use inherit_methods_macro::inherit_methods;
use objc2::{
    DeclaredClass, MainThreadOnly, define_class, msg_send,
    rc::{Allocated, Retained},
    runtime::ProtocolObject,
    sel,
};
use objc2_foundation::{MainThreadMarker, NSInteger, NSObject, NSObjectProtocol, NSString};
use objc2_ui_kit::{
    UIControlEvents, UIModalPresentationStyle, UIPickerView, UIPickerViewDataSource,
    UIPickerViewDelegate, UIPopoverArrowDirection, UITextBorderStyle, UITextField,
    UIViewController,
};
use winio_callback::Callback;
use winio_handle::AsContainer;
use winio_primitive::{Point, Size};

use crate::{GlobalRuntime, Result, catch, first_ui_window_scene, ui::Widget};

#[derive(Debug)]
pub struct ComboBox {
    handle: Widget,
    view: Retained<UITextField>,
    picker: Retained<UIPickerView>,
    delegate: Retained<ComboBoxDelegate>,
}

#[inherit_methods(from = "self.handle")]
impl ComboBox {
    pub fn new(parent: impl AsContainer) -> Result<Self> {
        let parent = parent.as_container();
        let mtm = parent.as_ui_kit().mtm();

        catch(|| unsafe {
            let view = UITextField::new(mtm);
            view.setBorderStyle(UITextBorderStyle::RoundedRect);

            let picker = UIPickerView::new(mtm);

            let delegate = ComboBoxDelegate::new(mtm);
            let del_obj = ProtocolObject::from_ref(&*delegate);
            picker.setDelegate(Some(del_obj));
            let del_obj = ProtocolObject::from_ref(&*delegate);
            picker.setDataSource(Some(del_obj));

            view.setInputView(Some(&picker));

            if cfg!(target_abi = "macabi") {
                view.addTarget_action_forControlEvents(
                    Some(&delegate),
                    sel!(showInputView:),
                    UIControlEvents::EditingDidBegin,
                );
            }

            let handle = Widget::from_uiview(parent, Retained::cast_unchecked(view.clone()))?;

            Ok(Self {
                handle,
                view,
                picker,
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

    pub fn text(&self) -> Result<String> {
        Ok(self
            .selection()?
            .map(|i| self.delegate.ivars().items.borrow()[i].clone())
            .unwrap_or_default())
    }

    pub fn set_text(&mut self, s: impl AsRef<str>) -> Result<()> {
        catch(|| self.view.setText(Some(&NSString::from_str(s.as_ref()))))
    }

    pub fn selection(&self) -> Result<Option<usize>> {
        let len = self.len()?;
        catch(|| {
            let index = self.picker.selectedRowInComponent(0);
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

            self.view.setText(Some(&NSString::from_str(
                &self.delegate.ivars().items.borrow()[i],
            )));
        })
    }

    pub fn is_editable(&self) -> Result<bool> {
        catch(|| self.view.isEnabled())
    }

    pub fn set_editable(&mut self, v: bool) -> Result<()> {
        catch(|| self.view.setEnabled(v))
    }

    pub async fn wait_change(&self) {
        self.delegate.ivars().changed.wait().await
    }

    pub async fn wait_select(&self) {
        self.delegate.ivars().select.wait().await;

        if let Ok(Some(i)) = self.selection() {
            self.view.setText(Some(&NSString::from_str(
                &self.delegate.ivars().items.borrow()[i],
            )));
        }
    }

    pub fn len(&self) -> Result<usize> {
        Ok(self.delegate.ivars().items.borrow().len())
    }

    pub fn is_empty(&self) -> Result<bool> {
        Ok(self.len()? == 0)
    }

    pub fn clear(&mut self) -> Result<()> {
        self.delegate.ivars().items.borrow_mut().clear();
        catch(|| self.picker.reloadAllComponents())
    }

    pub fn get(&self, i: usize) -> Result<String> {
        Ok(self.delegate.ivars().items.borrow()[i].clone())
    }

    pub fn set(&mut self, i: usize, s: impl AsRef<str>) -> Result<()> {
        self.delegate.ivars().items.borrow_mut()[i] = s.as_ref().to_string();
        catch(|| self.picker.reloadAllComponents())
    }

    pub fn insert(&mut self, i: usize, s: impl AsRef<str>) -> Result<()> {
        self.delegate
            .ivars()
            .items
            .borrow_mut()
            .insert(i, s.as_ref().to_string());
        catch(|| self.picker.reloadAllComponents())?;
        if !self.is_editable()? && self.len()? == 1 {
            self.set_selection(0)?;
        }
        Ok(())
    }

    pub fn remove(&mut self, i: usize) -> Result<()> {
        self.delegate.ivars().items.borrow_mut().remove(i);
        catch(|| self.picker.reloadAllComponents())
    }
}

winio_handle::impl_as_widget!(ComboBox, handle);

#[derive(Debug, Default)]
struct ComboBoxDelegateIvars {
    changed: Callback,
    select: Callback,
    items: RefCell<Vec<String>>,
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

        #[unsafe(method(showInputView:))]
        fn showInputView(&self, sender: &UITextField) {
            let input_view = sender.inputView();

            if let Some(input_view) = input_view {
                let vc = UIViewController::new(input_view.mtm());
                vc.setView(Some(&input_view));
                vc.setModalPresentationStyle(UIModalPresentationStyle::Popover);

                if let Some(popover) = vc.popoverPresentationController() {
                    popover.setSourceView(Some(sender));
                    popover.setSourceRect(sender.bounds());
                    popover.setPermittedArrowDirections(UIPopoverArrowDirection::Any);
                }

                if let Ok(Some(scene)) = first_ui_window_scene()
                    && let Some(window) = scene.keyWindow()
                    && let Some(controller) = window.rootViewController()
                {
                    controller.presentViewController_animated_completion(&vc, true, None);
                }
            }
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

        #[unsafe(method_id(pickerView:titleForRow:forComponent:))]
        fn pickerView_titleForRow_forComponent(
            &self,
            picker_view: &UIPickerView,
            row: NSInteger,
            _component: NSInteger,
        ) -> Option<Retained<NSString>> {
            let items = self.ivars().items.borrow();
            if row < 0 || (row as usize) >= items.len() {
                None
            } else {
                Some(NSString::from_str(&items[row as usize]))
            }
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
            self.ivars().items.borrow().len() as isize
        }
    }
}

impl ComboBoxDelegate {
    pub fn new(mtm: MainThreadMarker) -> Retained<Self> {
        unsafe { msg_send![mtm.alloc::<Self>(), init] }
    }
}
