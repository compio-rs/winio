use std::rc::Rc;

use gtk4::{
    glib::{self, object::Cast},
    prelude::ListModelExt,
    subclass::prelude::ObjectSubclassExt,
};
use inherit_methods_macro::inherit_methods;
use winio_callback::Callback;
use winio_handle::AsContainer;
use winio_primitive::{Point, Size};

use crate::{Error, GlobalRuntime, Result, ui::Widget};

#[derive(Debug)]
pub struct ComboBox {
    on_changed: Rc<Callback<()>>,
    model: StringListModel,
    widget: gtk4::DropDown,
    handle: Widget,
}

#[inherit_methods(from = "self.handle")]
impl ComboBox {
    pub fn new(parent: impl AsContainer) -> Result<Self> {
        let model = StringListModel::new();
        let widget = gtk4::DropDown::new(Some(model.clone()), gtk4::Expression::NONE);
        let handle = Widget::new(parent, unsafe { widget.clone().unsafe_cast() })?;
        let on_changed = Rc::new(Callback::new());
        widget.connect_selected_notify({
            let on_changed = on_changed.clone();
            move |_| {
                on_changed.signal::<GlobalRuntime>(());
            }
        });
        Ok(Self {
            on_changed,
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

    pub fn loc(&self) -> Result<Point>;

    pub fn set_loc(&mut self, p: Point) -> Result<()>;

    pub fn size(&self) -> Result<Size>;

    pub fn set_size(&mut self, s: Size) -> Result<()>;

    pub fn tooltip(&self) -> Result<String>;

    pub fn set_tooltip(&mut self, s: impl AsRef<str>) -> Result<()>;

    pub fn text(&self) -> Result<String> {
        match self.widget.selected_item() {
            Some(obj) => Ok(obj
                .downcast::<gtk4::StringObject>()
                .map_err(|_| Error::CastFailed)?
                .string()
                .to_string()),
            None => Ok(String::new()),
        }
    }

    pub fn set_text(&mut self, _s: impl AsRef<str>) -> Result<()> {
        self.handle.reset_preferred_size();
        Ok(())
    }

    pub fn selection(&self) -> Result<Option<usize>> {
        let index = self.widget.selected();
        if index == gtk4::INVALID_LIST_POSITION {
            Ok(None)
        } else {
            Ok(Some(index as _))
        }
    }

    pub fn set_selection(&mut self, i: usize) -> Result<()> {
        self.widget.set_selected(i as _);
        Ok(())
    }

    pub fn is_editable(&self) -> Result<bool> {
        Ok(self.widget.enables_search())
    }

    pub fn set_editable(&mut self, v: bool) -> Result<()> {
        self.widget.set_enable_search(v);
        Ok(())
    }

    pub async fn wait_change(&self) {
        self.on_changed.wait().await
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

        pub fn remove(&self, i: usize) -> Option<()> {
            let mut strings = self.strings.borrow_mut();
            if i >= strings.len() {
                return None;
            }
            strings.remove(i);
            self.obj().items_changed(i as _, 1, 0);
            Some(())
        }

        pub fn get(&self, i: usize) -> Option<String> {
            self.strings.borrow().get(i).cloned()
        }

        pub fn set(&self, i: usize, s: String) -> Option<()> {
            *self.strings.borrow_mut().get_mut(i)? = s;
            self.obj().items_changed(i as _, 1, 1);
            Some(())
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

    pub fn get(&self, i: usize) -> Result<String> {
        imp::StringListModel::from_obj(self)
            .get(i)
            .ok_or(Error::Index(i))
    }

    pub fn set(&self, i: usize, s: String) -> Result<()> {
        imp::StringListModel::from_obj(self)
            .set(i, s)
            .ok_or(Error::Index(i))
    }

    pub fn insert(&self, i: usize, s: String) {
        imp::StringListModel::from_obj(self).insert(i, s);
    }

    pub fn remove(&self, i: usize) -> Result<()> {
        imp::StringListModel::from_obj(self)
            .remove(i)
            .ok_or(Error::Index(i))
    }
}
