use std::rc::Rc;

use gtk4::{
    glib::{self, object::Cast},
    prelude::ListModelExt,
    subclass::prelude::ObjectSubclassExt,
};
use inherit_methods_macro::inherit_methods;
use winio_callback::Callback;
use winio_handle::AsWindow;
use winio_primitive::{Point, Size};

use crate::{GlobalRuntime, ui::Widget};

#[derive(Debug)]
pub struct ComboBox {
    on_changed: Rc<Callback<()>>,
    model: StringListModel,
    widget: gtk4::DropDown,
    handle: Widget,
}

#[inherit_methods(from = "self.handle")]
impl ComboBox {
    pub fn new(parent: impl AsWindow) -> Self {
        let model = StringListModel::new();
        let widget = gtk4::DropDown::new(Some(model.clone()), gtk4::Expression::NONE);
        let handle = Widget::new(parent, unsafe { widget.clone().unsafe_cast() });
        let on_changed = Rc::new(Callback::new());
        widget.connect_selected_notify({
            let on_changed = Rc::downgrade(&on_changed);
            move |_| {
                if let Some(on_changed) = on_changed.upgrade() {
                    on_changed.signal::<GlobalRuntime>(());
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

    pub fn is_visible(&self) -> bool;

    pub fn set_visible(&mut self, v: bool);

    pub fn is_enabled(&self) -> bool;

    pub fn set_enabled(&mut self, v: bool);

    pub fn preferred_size(&self) -> Size;

    pub fn loc(&self) -> Point;

    pub fn set_loc(&mut self, p: Point);

    pub fn size(&self) -> Size;

    pub fn set_size(&mut self, s: Size);

    pub fn text(&self) -> String {
        self.widget
            .selected_item()
            .map(|obj| {
                obj.downcast::<gtk4::StringObject>()
                    .unwrap()
                    .string()
                    .to_string()
            })
            .unwrap_or_default()
    }

    pub fn set_text(&mut self, _s: impl AsRef<str>) {
        self.handle.reset_preferred_size();
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

    pub fn is_editable(&self) -> bool {
        self.widget.enables_search()
    }

    pub fn set_editable(&mut self, v: bool) {
        self.widget.set_enable_search(v);
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

winio_handle::impl_as_widget!(ComboBox, handle);

mod imp {
    use std::cell::RefCell;

    use gtk4::{
        glib,
        prelude::{Cast, ListModelExt},
        subclass::prelude::*,
    };

    #[derive(Debug, Default)]
    pub struct StringListModel {
        strings: RefCell<Vec<String>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for StringListModel {
        type Interfaces = (gtk4::gio::ListModel,);
        type ParentType = glib::Object;
        type Type = super::StringListModel;

        const ABSTRACT: bool = false;
        const NAME: &'static str = "StringListModel";
    }

    impl ObjectImpl for StringListModel {}

    impl ListModelImpl for StringListModel {
        fn item_type(&self) -> glib::Type {
            glib::Type::OBJECT
        }

        fn n_items(&self) -> u32 {
            self.strings.borrow().len() as _
        }

        fn item(&self, position: u32) -> Option<glib::Object> {
            if position >= self.n_items() {
                return None;
            }
            Some(gtk4::StringObject::new(&self.strings.borrow()[position as usize]).upcast())
        }
    }

    impl StringListModel {
        pub fn insert(&self, i: usize, s: String) {
            self.strings.borrow_mut().insert(i, s);
            self.obj().items_changed(i as _, 0, 1);
        }

        pub fn remove(&self, i: usize) {
            self.strings.borrow_mut().remove(i);
            self.obj().items_changed(i as _, 1, 0);
        }

        pub fn get(&self, i: usize) -> String {
            self.strings.borrow()[i].to_string()
        }

        pub fn set(&self, i: usize, s: String) {
            self.strings.borrow_mut()[i] = s;
            self.obj().items_changed(i as _, 1, 1);
        }

        pub fn clear(&self) {
            let len = {
                let mut strings = self.strings.borrow_mut();
                let len = strings.len();
                strings.clear();
                len
            };
            self.obj().items_changed(0, len as _, 0);
        }
    }
}

glib::wrapper! {
    pub struct StringListModel(ObjectSubclass<imp::StringListModel>)
        @implements gtk4::gio::ListModel;
}

impl Default for StringListModel {
    fn default() -> Self {
        Self::new()
    }
}

impl StringListModel {
    pub fn new() -> Self {
        glib::Object::new()
    }

    pub fn clear(&self) {
        imp::StringListModel::from_obj(self).clear();
    }

    pub fn get(&self, i: usize) -> String {
        imp::StringListModel::from_obj(self).get(i)
    }

    pub fn set(&self, i: usize, s: String) {
        imp::StringListModel::from_obj(self).set(i, s);
    }

    pub fn insert(&self, i: usize, s: String) {
        imp::StringListModel::from_obj(self).insert(i, s);
    }

    pub fn remove(&self, i: usize) {
        imp::StringListModel::from_obj(self).remove(i);
    }
}
