use std::rc::Rc;

use gtk4::{glib::object::Cast, prelude::WidgetExt};
use inherit_methods_macro::inherit_methods;
use winio_callback::Callback;
use winio_handle::{AsContainer, AsRawContainer, RawContainer};
use winio_primitive::{Point, Size};

use crate::{GlobalRuntime, Widget};

#[derive(Debug)]
pub struct TabView {
    on_select: Rc<Callback>,
    view: gtk4::Notebook,
    handle: Widget,
}

#[inherit_methods(from = "self.handle")]
impl TabView {
    pub fn new(parent: impl AsContainer) -> Self {
        let view = gtk4::Notebook::new();
        view.set_scrollable(true);
        let handle = Widget::new(parent, unsafe { view.clone().unsafe_cast() });
        let on_select = Rc::new(Callback::new());
        view.connect_select_page({
            let on_select = on_select.clone();
            move |_, _| {
                on_select.signal::<GlobalRuntime>(());
                true
            }
        });
        Self {
            on_select,
            view,
            handle,
        }
    }

    pub fn is_visible(&self) -> bool;

    pub fn set_visible(&mut self, v: bool);

    pub fn is_enabled(&self) -> bool;

    pub fn set_enabled(&mut self, v: bool);

    pub fn loc(&self) -> Point;

    pub fn set_loc(&mut self, p: Point);

    pub fn size(&self) -> Size;

    pub fn set_size(&mut self, s: Size);

    pub fn selection(&self) -> Option<usize> {
        self.view.current_page().map(|i| i as usize)
    }

    pub fn set_selection(&mut self, i: Option<usize>) {
        self.view.set_current_page(i.map(|i| i as _));
    }

    pub async fn wait_select(&self) {
        self.on_select.wait().await
    }

    pub fn insert(&mut self, i: usize, item: &TabViewItem) {
        self.view
            .insert_page(&item.swindow, Some(&item.label), Some(i as _));
    }

    pub fn remove(&mut self, i: usize) {
        self.view.remove_page(Some(i as _));
    }

    pub fn len(&self) -> usize {
        self.view.n_pages() as _
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn clear(&mut self) {
        while !self.is_empty() {
            self.remove(0);
        }
    }
}

winio_handle::impl_as_widget!(TabView, handle);

#[derive(Debug)]
pub struct TabViewItem {
    swindow: gtk4::ScrolledWindow,
    fixed: gtk4::Fixed,
    label: gtk4::Label,
}

impl TabViewItem {
    pub fn new(_parent: &TabView) -> Self {
        let swindow = gtk4::ScrolledWindow::new();
        swindow.set_hscrollbar_policy(gtk4::PolicyType::External);
        swindow.set_vscrollbar_policy(gtk4::PolicyType::External);
        let fixed = gtk4::Fixed::new();
        swindow.set_child(Some(&fixed));
        let label = gtk4::Label::new(None);
        Self {
            swindow,
            fixed,
            label,
        }
    }

    pub fn text(&self) -> String {
        self.label.text().to_string()
    }

    pub fn set_text(&mut self, s: impl AsRef<str>) {
        self.label.set_text(s.as_ref());
    }

    pub fn size(&self) -> Size {
        let width = self.swindow.width();
        let height = self.swindow.height();
        Size::new(width as _, height as _)
    }
}

impl AsRawContainer for TabViewItem {
    fn as_raw_container(&self) -> RawContainer {
        RawContainer::Gtk(self.fixed.clone())
    }
}

winio_handle::impl_as_container!(TabViewItem);
