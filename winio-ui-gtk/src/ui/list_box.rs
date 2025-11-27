use std::rc::Rc;

use gtk4::{
    gio::prelude::ListModelExt,
    glib::object::Cast,
    prelude::{ListBoxRowExt, WidgetExt},
};
use inherit_methods_macro::inherit_methods;
use winio_callback::Callback;
use winio_handle::AsContainer;
use winio_primitive::{Point, Size};

use crate::{
    Error, GlobalRuntime, Result,
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
    pub fn new(parent: impl AsContainer) -> Result<Self> {
        let swindow = gtk4::ScrolledWindow::new();
        swindow.set_hscrollbar_policy(gtk4::PolicyType::Never);
        let model = StringListModel::new();
        let widget = gtk4::ListBox::new();
        widget.bind_model(Some(&model), |obj| {
            let text = obj
                .downcast_ref::<gtk4::StringObject>()
                .expect("Failed to downcast to StringObject")
                .string();
            let label = gtk4::Label::new(Some(&text));
            label.set_halign(gtk4::Align::Start);
            unsafe { label.unsafe_cast() }
        });
        widget.set_selection_mode(gtk4::SelectionMode::Multiple);
        swindow.set_child(Some(&widget));
        let handle = Widget::new(parent, unsafe { swindow.clone().unsafe_cast() })?;
        let on_changed = Rc::new(Callback::new());
        widget.connect_selected_rows_changed({
            let on_changed = on_changed.clone();
            move |_| {
                on_changed.signal::<GlobalRuntime>(());
            }
        });
        Ok(Self {
            on_changed,
            swindow,
            model,
            widget,
            handle,
        })
    }

    pub fn is_visible(&self) -> Result<bool>;

    pub fn set_visible(&mut self, v: bool) -> Result<()>;

    pub fn is_enabled(&self) -> Result<bool>;

    pub fn set_enabled(&mut self, v: bool) -> Result<()>;

    pub fn preferred_size(&self) -> Result<Size>;

    pub fn min_size(&self) -> Result<Size> {
        let size = self.preferred_size()?;
        Ok(Size::new(size.width, 0.0))
    }

    pub fn loc(&self) -> Result<Point>;

    pub fn set_loc(&mut self, p: Point) -> Result<()>;

    pub fn size(&self) -> Result<Size>;

    pub fn set_size(&mut self, s: Size) -> Result<()>;

    pub fn tooltip(&self) -> Result<String>;

    pub fn set_tooltip(&mut self, s: impl AsRef<str>) -> Result<()>;

    pub fn is_selected(&self, i: usize) -> Result<bool> {
        Ok(self
            .widget
            .row_at_index(i as _)
            .ok_or(Error::Index(i))?
            .is_selected())
    }

    pub fn set_selected(&mut self, i: usize, v: bool) -> Result<()> {
        let row = self.widget.row_at_index(i as _).ok_or(Error::Index(i))?;
        if v {
            self.widget.select_row(Some(&row));
        } else {
            self.widget.unselect_row(&row);
        }
        Ok(())
    }

    pub async fn wait_select(&self) {
        self.on_changed.wait().await
    }

    pub fn len(&self) -> Result<usize> {
        Ok(self.model.n_items() as _)
    }

    pub fn is_empty(&self) -> Result<bool> {
        Ok(self.len()? == 0)
    }

    pub fn clear(&mut self) -> Result<()> {
        self.model.clear();
        self.handle.reset_preferred_size();
        Ok(())
    }

    pub fn get(&self, i: usize) -> Result<String> {
        self.model.get(i)
    }

    pub fn set(&mut self, i: usize, s: impl AsRef<str>) -> Result<()> {
        self.model.set(i, s.as_ref().to_string())?;
        self.handle.reset_preferred_size();
        Ok(())
    }

    pub fn insert(&mut self, i: usize, s: impl AsRef<str>) -> Result<()> {
        self.model.insert(i, s.as_ref().to_string());
        self.handle.reset_preferred_size();
        Ok(())
    }

    pub fn remove(&mut self, i: usize) -> Result<()> {
        self.model.remove(i as _)?;
        self.handle.reset_preferred_size();
        Ok(())
    }
}

winio_handle::impl_as_widget!(ListBox, handle);
