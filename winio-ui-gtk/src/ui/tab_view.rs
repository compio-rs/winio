use std::rc::Rc;

use gtk4::{glib::object::Cast, prelude::WidgetExt};
use inherit_methods_macro::inherit_methods;
use winio_callback::Callback;
use winio_handle::{AsContainer, AsRawContainer, RawContainer};
use winio_primitive::{Point, Size};

use crate::{GlobalRuntime, Result, Widget};

#[derive(Debug)]
pub struct TabView {
    on_select: Rc<Callback>,
    view: gtk4::Notebook,
    handle: Widget,
}

#[inherit_methods(from = "self.handle")]
impl TabView {
    pub fn new(parent: impl AsContainer) -> Result<Self> {
        let view = gtk4::Notebook::new();
        view.set_scrollable(true);
        let handle = Widget::new(parent, unsafe { view.clone().unsafe_cast() })?;
        let on_select = Rc::new(Callback::new());
        view.connect_select_page({
            let on_select = on_select.clone();
            move |_, _| {
                on_select.signal::<GlobalRuntime>(());
                true
            }
        });
        Ok(Self {
            on_select,
            view,
            handle,
        })
    }

    pub fn is_visible(&self) -> Result<bool>;

    pub fn set_visible(&mut self, v: bool) -> Result<()>;

    pub fn is_enabled(&self) -> Result<bool>;

    pub fn set_enabled(&mut self, v: bool) -> Result<()>;

    pub fn loc(&self) -> Result<Point>;

    pub fn set_loc(&mut self, p: Point) -> Result<()>;

    pub fn size(&self) -> Result<Size>;

    pub fn set_size(&mut self, s: Size) -> Result<()>;

    pub fn selection(&self) -> Result<Option<usize>> {
        Ok(self.view.current_page().map(|i| i as usize))
    }

    pub fn set_selection(&mut self, i: usize) -> Result<()> {
        self.view.set_current_page(Some(i as _));
        Ok(())
    }

    pub async fn wait_select(&self) {
        self.on_select.wait().await
    }

    pub fn insert(&mut self, i: usize, item: &TabViewItem) -> Result<()> {
        self.view
            .insert_page(&item.swindow, Some(&item.label), Some(i as _));
        Ok(())
    }

    pub fn remove(&mut self, i: usize) -> Result<()> {
        self.view.remove_page(Some(i as _));
        Ok(())
    }

    pub fn len(&self) -> Result<usize> {
        Ok(self.view.n_pages() as _)
    }

    pub fn is_empty(&self) -> Result<bool> {
        Ok(self.len()? == 0)
    }

    pub fn clear(&mut self) -> Result<()> {
        while !self.is_empty()? {
            self.remove(0)?;
        }
        Ok(())
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
