use std::rc::Rc;

use gtk4::{
    gio::prelude::ListModelExt,
    glib::object::Cast,
    prelude::{ListBoxRowExt, WidgetExt},
};
use inherit_methods_macro::inherit_methods;
use winio_callback::Callback;
use winio_handle::AsWindow;
use winio_primitive::{Point, Size};

use crate::{
    GlobalRuntime,
    ui::{StringListModel, Widget},
};

#[derive(Debug)]
pub struct ListBox {
    on_changed: Rc<Callback<()>>,
    #[allow(dead_code)]
    swindow: gtk4::ScrolledWindow,
    model: StringListModel,
    widget: gtk4::ListBox,
    handle: Widget,
}

#[inherit_methods(from = "self.handle")]
impl ListBox {
    pub fn new(parent: impl AsWindow) -> Self {
        let swindow = gtk4::ScrolledWindow::new();
        swindow.set_hscrollbar_policy(gtk4::PolicyType::Never);
        let model = StringListModel::new();
        let widget = gtk4::ListBox::new();
        widget.bind_model(Some(&model), |obj| {
            let text = obj.downcast_ref::<gtk4::StringObject>().unwrap().string();
            let label = gtk4::Label::new(Some(&text));
            label.set_halign(gtk4::Align::Start);
            unsafe { label.unsafe_cast() }
        });
        widget.set_selection_mode(gtk4::SelectionMode::Multiple);
        swindow.set_child(Some(&widget));
        let handle = Widget::new(parent, unsafe { swindow.clone().unsafe_cast() });
        let on_changed = Rc::new(Callback::new());
        widget.connect_selected_rows_changed({
            let on_changed = on_changed.clone();
            move |_| {
                on_changed.signal::<GlobalRuntime>(());
            }
        });
        Self {
            on_changed,
            swindow,
            model,
            widget,
            handle,
        }
    }

    pub fn is_visible(&self) -> bool;

    pub fn set_visible(&mut self, v: bool);

    pub fn is_enabled(&self) -> bool;

    pub fn set_enabled(&mut self, v: bool);

    pub fn preferred_size(&self) -> Size;

    pub fn min_size(&self) -> Size {
        let size = self.preferred_size();
        Size::new(size.width, 0.0)
    }

    pub fn loc(&self) -> Point;

    pub fn set_loc(&mut self, p: Point);

    pub fn size(&self) -> Size;

    pub fn set_size(&mut self, s: Size);

    pub fn is_selected(&self, i: usize) -> bool {
        self.widget
            .row_at_index(i as _)
            .map(|row| row.is_selected())
            .unwrap_or_default()
    }

    pub fn set_selected(&mut self, i: usize, v: bool) {
        if let Some(row) = self.widget.row_at_index(i as _) {
            if v {
                self.widget.select_row(Some(&row));
            } else {
                self.widget.unselect_row(&row);
            }
        }
    }

    pub async fn wait_select(&self) {
        self.on_changed.wait().await
    }

    pub fn len(&self) -> usize {
        self.model.n_items() as _
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn clear(&mut self) {
        self.model.clear();
        self.handle.reset_preferred_size();
    }

    pub fn get(&self, i: usize) -> String {
        self.model.get(i)
    }

    pub fn set(&mut self, i: usize, s: impl AsRef<str>) {
        self.model.set(i, s.as_ref().to_string());
        self.handle.reset_preferred_size();
    }

    pub fn insert(&mut self, i: usize, s: impl AsRef<str>) {
        self.model.insert(i, s.as_ref().to_string());
        self.handle.reset_preferred_size();
    }

    pub fn remove(&mut self, i: usize) {
        self.model.remove(i as _);
        self.handle.reset_preferred_size();
    }
}

winio_handle::impl_as_widget!(ListBox, handle);
