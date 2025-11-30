use inherit_methods_macro::inherit_methods;
use winio_elm::{Component, ComponentSender};
use winio_handle::BorrowedContainer;
use winio_primitive::{Enable, Failable, Layoutable, Point, Size, TextWidget, Visible};

use crate::{
    sys,
    sys::{Error, Result},
};

/// A tabbed view that contains many [`TabViewItem`]s.
#[derive(Debug)]
pub struct TabView {
    widget: sys::TabView,
}

impl Failable for TabView {
    type Error = Error;
}

#[inherit_methods(from = "self.widget")]
impl TabView {
    /// The selection index.
    pub fn selection(&self) -> Result<Option<usize>>;

    /// Set the selection.
    pub fn set_selection(&mut self, i: usize) -> Result<()>;

    /// Insert a new tab item.
    pub fn insert(&mut self, i: usize, item: &TabViewItem) -> Result<()> {
        self.widget.insert(i, &item.widget)
    }

    /// Push a new tab item to the end.
    pub fn push(&mut self, item: &TabViewItem) -> Result<()> {
        self.insert(self.len()?, item)
    }

    /// Remove a tab by index.
    pub fn remove(&mut self, i: usize) -> Result<()>;

    /// The length of the tabs.
    pub fn len(&self) -> Result<usize>;

    /// If the tab collection is empty.
    pub fn is_empty(&self) -> Result<bool>;

    /// Clear the tabs.
    pub fn clear(&mut self) -> Result<()>;
}

#[inherit_methods(from = "self.widget")]
impl Visible for TabView {
    fn is_visible(&self) -> Result<bool>;

    fn set_visible(&mut self, v: bool) -> Result<()>;
}

#[inherit_methods(from = "self.widget")]
impl Enable for TabView {
    fn is_enabled(&self) -> Result<bool>;

    fn set_enabled(&mut self, v: bool) -> Result<()>;
}

#[inherit_methods(from = "self.widget")]
impl Layoutable for TabView {
    fn loc(&self) -> Result<Point>;

    fn set_loc(&mut self, p: Point) -> Result<()>;

    fn size(&self) -> Result<Size>;

    fn set_size(&mut self, v: Size) -> Result<()>;
}

/// Events of [`TabView`].
#[derive(Debug)]
#[non_exhaustive]
pub enum TabViewEvent {
    /// The selection changed.
    Select,
}

/// Messages of [`TabView`].
#[derive(Debug)]
#[non_exhaustive]
pub enum TabViewMessage {}

impl Component for TabView {
    type Error = Error;
    type Event = TabViewEvent;
    type Init<'a> = BorrowedContainer<'a>;
    type Message = TabViewMessage;

    fn init(init: Self::Init<'_>, _sender: &ComponentSender<Self>) -> Result<Self> {
        let widget = sys::TabView::new(init)?;
        Ok(Self { widget })
    }

    async fn start(&mut self, sender: &ComponentSender<Self>) -> ! {
        loop {
            self.widget.wait_select().await;
            sender.output(TabViewEvent::Select);
        }
    }
}

winio_handle::impl_as_widget!(TabView, widget);

/// A tab item of [`TabView`].
#[derive(Debug)]
pub struct TabViewItem {
    widget: sys::TabViewItem,
}

impl Failable for TabViewItem {
    type Error = Error;
}

#[inherit_methods(from = "self.widget")]
impl TextWidget for TabViewItem {
    fn text(&self) -> Result<String>;

    fn set_text(&mut self, s: impl AsRef<str>) -> Result<()>;
}

#[inherit_methods(from = "self.widget")]
impl TabViewItem {
    /// Get the available size of the tab.
    pub fn size(&self) -> Result<Size>;
}

/// Events of [`TabViewItem`].
#[derive(Debug)]
#[non_exhaustive]
pub enum TabViewItemEvent {}

/// Messages of [`TabViewItem`].
#[derive(Debug)]
#[non_exhaustive]
pub enum TabViewItemMessage {}

impl Component for TabViewItem {
    type Error = Error;
    type Event = TabViewItemEvent;
    type Init<'a> = &'a TabView;
    type Message = TabViewItemMessage;

    fn init(init: Self::Init<'_>, _sender: &ComponentSender<Self>) -> Result<Self> {
        let widget = sys::TabViewItem::new(&init.widget)?;
        Ok(Self { widget })
    }
}

winio_handle::impl_as_container!(TabViewItem, widget);
