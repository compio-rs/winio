use std::{cell::RefCell, rc::Rc};

use objc2::{MainThreadMarker, MainThreadOnly, rc::Retained};
use objc2_foundation::NSArray;
use objc2_ui_kit::{NSLayoutConstraint, UIBarButtonItem, UIToolbar};
use winio_handle::AsWindow;

use crate::{Error, Result, catch, runtime::signal_resize, widgets::ToolBarHost};

#[derive(Debug)]
pub(crate) struct ToolBarData {
    toolbar: Retained<UIToolbar>,
    host: ToolBarHost,
    grow: Retained<NSLayoutConstraint>,
    items: RefCell<Vec<(usize, Retained<UIBarButtonItem>)>>,
}

impl ToolBarData {
    pub(crate) fn add_item(&self, id: usize, item: Retained<UIBarButtonItem>) {
        self.items.borrow_mut().push((id, item));
        self.sync();
    }

    pub(crate) fn remove_item(&self, id: usize) {
        self.items
            .borrow_mut()
            .retain(|(item_id, _)| *item_id != id);
        self.sync();
    }

    fn sync(&self) {
        let items: Vec<Retained<UIBarButtonItem>> = self
            .items
            .borrow()
            .iter()
            .map(|(_, item)| item.clone())
            .collect();
        self.toolbar
            .setItems(Some(&NSArray::from_retained_slice(&items)));
    }

    pub(crate) fn mtm(&self) -> MainThreadMarker {
        self.toolbar.mtm()
    }
}

#[derive(Debug)]
pub struct ToolBar {
    data: Rc<ToolBarData>,
}

impl ToolBar {
    pub fn new(parent: impl AsWindow) -> Result<Self> {
        let wnd = parent.as_window().as_ui_kit();
        let mtm = wnd.mtm();
        let host = ToolBarHost::of(wnd)?;
        if !host.view.subviews().is_empty() {
            return Err(Error::ToolBarExists);
        }

        catch(|| {
            let toolbar = UIToolbar::new(mtm);
            toolbar.setTranslatesAutoresizingMaskIntoConstraints(false);
            host.view.addSubview(&toolbar);

            let a1 = toolbar
                .leadingAnchor()
                .constraintEqualToAnchor(&host.view.leadingAnchor());
            let a2 = toolbar
                .trailingAnchor()
                .constraintEqualToAnchor(&host.view.trailingAnchor());
            let a3 = toolbar
                .topAnchor()
                .constraintEqualToAnchor(&host.view.topAnchor());
            let a4 = toolbar
                .bottomAnchor()
                .constraintEqualToAnchor(&host.view.bottomAnchor());
            let grow = host.view.heightAnchor().constraintEqualToConstant(44.0);
            NSLayoutConstraint::activateConstraints(
                &NSArray::from_retained_slice(&[a1, a2, a3, a4]),
                mtm,
            );
            host.zero_height.setActive(false);
            grow.setActive(true);

            Ok(Self {
                data: Rc::new(ToolBarData {
                    toolbar,
                    host,
                    grow,
                    items: RefCell::new(Vec::new()),
                }),
            })
        })
        .flatten()
    }

    pub fn is_visible(&self) -> Result<bool> {
        catch(|| !self.data.host.view.isHidden())
    }

    pub fn set_visible(&mut self, visible: bool) -> Result<()> {
        if visible == self.is_visible()? {
            return Ok(());
        }
        catch(|| {
            self.data.host.view.setHidden(!visible);
            if visible {
                self.data.host.zero_height.setActive(false);
                self.data.grow.setActive(true);
            } else {
                self.data.grow.setActive(false);
                self.data.host.zero_height.setActive(true);
            }
            if let Some(parent) = self.data.host.view.superview() {
                parent.layoutIfNeeded();
            }
        })?;
        let _ = signal_resize::<()>();
        Ok(())
    }

    pub(crate) fn data(&self) -> &Rc<ToolBarData> {
        &self.data
    }
}

impl Drop for ToolBar {
    fn drop(&mut self) {
        self.data.grow.setActive(false);
        self.data.host.zero_height.setActive(true);
        self.data.toolbar.removeFromSuperview();
        if let Some(parent) = self.data.host.view.superview() {
            parent.layoutIfNeeded();
        }
        let _ = signal_resize::<()>();
    }
}
