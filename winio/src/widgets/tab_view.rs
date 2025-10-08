use inherit_methods_macro::inherit_methods;
use winio_elm::{Component, ComponentSender};
use winio_handle::BorrowedContainer;
use winio_layout::{Enable, Layoutable, TextWidget, Visible};
use winio_primitive::{Point, Size};

use crate::sys;

/// A tabbed view that contains many [`TabViewItem`]s.
#[derive(Debug)]
pub struct TabView {
    widget: sys::TabView,
}

#[inherit_methods(from = "self.widget")]
impl TabView {
    /// The selection index.
    pub fn selection(&self) -> Option<usize>;

    /// Set the selection.
    pub fn set_selection(&mut self, i: usize);

    /// Insert a new tab item.
    pub fn insert(&mut self, i: usize, item: &TabViewItem) {
        self.widget.insert(i, &item.widget)
    }

    /// Append a new tab item.
    pub fn append(&mut self, item: &TabViewItem) {
        self.insert(self.len(), item)
    }

    /// Remove a tab by index.
    pub fn remove(&mut self, i: usize);

    /// The length of the tabs.
    pub fn len(&self) -> usize;

    /// If the tab collection is empty.
    pub fn is_empty(&self) -> bool;

    /// Clear the tabs.
    pub fn clear(&mut self);
}

#[inherit_methods(from = "self.widget")]
impl Visible for TabView {
    fn is_visible(&self) -> bool;

    fn set_visible(&mut self, v: bool);
}

#[inherit_methods(from = "self.widget")]
impl Enable for TabView {
    fn is_enabled(&self) -> bool;

    fn set_enabled(&mut self, v: bool);
}

#[inherit_methods(from = "self.widget")]
impl Layoutable for TabView {
    fn loc(&self) -> Point;

    fn set_loc(&mut self, p: Point);

    fn size(&self) -> Size;

    fn set_size(&mut self, v: Size);
}

/// Events of [`TabView`].
#[non_exhaustive]
pub enum TabViewEvent {
    /// The selection changed.
    Select,
}

impl Component for TabView {
    type Event = TabViewEvent;
    type Init<'a> = BorrowedContainer<'a>;
    type Message = ();

    fn init(init: Self::Init<'_>, _sender: &ComponentSender<Self>) -> Self {
        let widget = sys::TabView::new(init);
        Self { widget }
    }

    async fn start(&mut self, sender: &ComponentSender<Self>) -> ! {
        loop {
            self.widget.wait_select().await;
            sender.output(TabViewEvent::Select);
        }
    }

    async fn update(&mut self, _message: Self::Message, _sender: &ComponentSender<Self>) -> bool {
        false
    }

    fn render(&mut self, _sender: &ComponentSender<Self>) {}
}

winio_handle::impl_as_widget!(TabView, widget);

/// A tab item of [`TabView`].
#[derive(Debug)]
pub struct TabViewItem {
    widget: sys::TabViewItem,
}

#[inherit_methods(from = "self.widget")]
impl TextWidget for TabViewItem {
    fn text(&self) -> String;

    fn set_text(&mut self, s: impl AsRef<str>);
}

#[inherit_methods(from = "self.widget")]
impl TabViewItem {
    /// Get the available size of the tab.
    pub fn size(&self) -> Size;
}

/// Events of [`TabViewItem`].
#[non_exhaustive]
pub enum TabViewItemEvent {}

impl Component for TabViewItem {
    type Event = TabViewItemEvent;
    type Init<'a> = &'a TabView;
    type Message = ();

    fn init(init: Self::Init<'_>, _sender: &ComponentSender<Self>) -> Self {
        let widget = sys::TabViewItem::new(&init.widget);
        Self { widget }
    }

    async fn start(&mut self, _sender: &ComponentSender<Self>) -> ! {
        std::future::pending().await
    }

    async fn update(&mut self, _message: Self::Message, _sender: &ComponentSender<Self>) -> bool {
        false
    }

    fn render(&mut self, _sender: &ComponentSender<Self>) {}
}

winio_handle::impl_as_container!(TabViewItem, widget);
