use std::rc::Rc;

use gtk4::{glib::object::Cast, prelude::WidgetExt};
use inherit_methods_macro::inherit_methods;
use winio_callback::Callback;
use winio_handle::{AsContainer, BorrowedContainer};
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
}

winio_handle::impl_as_widget!(TabView, handle);

#[derive(Debug)]
pub struct TabViewItem {
    parent: gtk4::glib::WeakRef<gtk4::Notebook>,
    swindow: gtk4::ScrolledWindow,
    fixed: gtk4::Fixed,
    label: gtk4::Label,
}

impl TabViewItem {
    pub fn new(parent: &TabView) -> Result<Self> {
        let swindow = gtk4::ScrolledWindow::new();
        swindow.set_hscrollbar_policy(gtk4::PolicyType::External);
        swindow.set_vscrollbar_policy(gtk4::PolicyType::External);
        let fixed = gtk4::Fixed::new();
        swindow.set_child(Some(&fixed));
        let label = gtk4::Label::new(None);
        parent.view.append_page(&swindow, Some(&label));
        let parent_weak = gtk4::glib::WeakRef::new();
        parent_weak.set(Some(&parent.view));
        Ok(Self {
            parent: parent_weak,
            swindow,
            fixed,
            label,
        })
    }

    pub fn text(&self) -> Result<String> {
        Ok(self.label.text().to_string())
    }

    pub fn set_text(&mut self, s: impl AsRef<str>) -> Result<()> {
        self.label.set_text(s.as_ref());
        Ok(())
    }

    pub fn size(&self) -> Result<Size> {
        let width = self.swindow.width();
        let height = self.swindow.height();
        Ok(Size::new(width as _, height as _))
    }
}

impl Drop for TabViewItem {
    fn drop(&mut self) {
        let Some(parent) = self.parent.upgrade() else {
            return;
        };
        if let Some(index) = parent.page_num(&self.swindow) {
            parent.remove_page(Some(index));
        }
    }
}

impl AsContainer for TabViewItem {
    fn as_container(&self) -> BorrowedContainer<'_> {
        BorrowedContainer::gtk(&self.fixed)
    }
}
