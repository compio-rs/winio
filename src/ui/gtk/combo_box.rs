use std::rc::Rc;

use gtk4::{glib::object::Cast, prelude::ListModelExt};

use crate::{
    AsWindow, Point, Size,
    ui::{Callback, Widget},
};

#[derive(Debug)]
pub struct ComboBoxImpl<const E: bool> {
    on_changed: Rc<Callback<()>>,
    model: gtk4::StringList,
    widget: gtk4::DropDown,
    handle: Widget,
}

impl<const E: bool> ComboBoxImpl<E> {
    pub fn new(parent: impl AsWindow) -> Self {
        let model = gtk4::StringList::new(&[]);
        let widget = gtk4::DropDown::new(Some(model.clone()), gtk4::Expression::NONE);
        if E {
            widget.set_enable_search(true);
        }
        let handle = Widget::new(parent, unsafe { widget.clone().unsafe_cast() });
        let on_changed = Rc::new(Callback::new());
        widget.connect_activate({
            let on_changed = Rc::downgrade(&on_changed);
            move |_| {
                if let Some(on_changed) = on_changed.upgrade() {
                    on_changed.signal(());
                }
            }
        });
        Self {
            on_changed,
            model,
            widget,
            handle,
        }
    }

    pub fn preferred_size(&self) -> Size {
        self.handle.preferred_size()
    }

    pub fn loc(&self) -> Point {
        self.handle.loc()
    }

    pub fn set_loc(&mut self, p: Point) {
        self.handle.set_loc(p);
    }

    pub fn size(&self) -> Size {
        self.handle.size()
    }

    pub fn set_size(&mut self, s: Size) {
        self.handle.set_size(s);
    }

    pub fn text(&self) -> String {
        todo!()
    }

    pub fn set_text(&mut self, s: impl AsRef<str>) {
        todo!()
    }

    pub fn selection(&self) -> Option<usize> {
        let index = self.widget.selected();
        if index == gtk4::INVALID_LIST_POSITION {
            None
        } else {
            Some(index as _)
        }
    }

    pub fn set_selection(&mut self, i: Option<usize>) {
        self.widget
            .set_selected(i.map(|i| i as u32).unwrap_or(gtk4::INVALID_LIST_POSITION));
    }

    pub async fn wait_change(&self) {
        self.on_changed.wait().await
    }

    pub async fn wait_select(&self) {
        self.on_changed.wait().await
    }

    pub fn len(&self) -> usize {
        self.model.n_items() as _
    }

    pub fn clear(&mut self) {
        while self.len() > 0 {
            self.model.remove(0);
        }
    }

    pub fn get(&self, i: usize) -> String {
        self.model
            .string(i as _)
            .map(|s| s.to_string())
            .unwrap_or_default()
    }

    pub fn set(&mut self, i: usize, s: impl AsRef<str>) {
        todo!()
    }

    pub fn insert(&mut self, i: usize, s: impl AsRef<str>) {
        self.model.append(s.as_ref());
    }

    pub fn remove(&mut self, i: usize) {
        self.model.remove(i as _);
    }
}

pub type ComboBox = ComboBoxImpl<false>;
pub type ComboEntry = ComboBoxImpl<true>;
