use inherit_methods_macro::inherit_methods;
use winio_elm::{Child, Component, ComponentSender, Prop, PropSinkEvent, PropSinkMessage, start};
use winio_handle::BorrowedContainer;
use winio_primitive::{Enable, Failable, Layoutable, Point, Rect, Size, TextWidget, Visible};

use crate::{
    sys,
    sys::{Error, Result},
};

/// A tabbed view that contains many [`TabViewItem`]s.
#[derive(Debug)]
pub struct TabView {
    widget: sys::TabView,
    selection_prop: Child<Prop<Option<usize>>>,
}

impl Failable for TabView {
    type Error = Error;
}

#[inherit_methods(from = "self.widget")]
impl TabView {
    /// The selection index.
    pub fn selection(&self) -> Result<Option<usize>>;

    /// Set the selection.
    pub fn set_selection(&mut self, i: usize) -> Result<()> {
        self.selection_prop.set(Some(i));
        Ok(())
    }

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

    /// Property for [`TabView::selection`].
    pub fn selection_prop(&mut self) -> &mut Prop<Option<usize>> {
        &mut self.selection_prop
    }
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

    fn set_size(&mut self, s: Size) -> Result<()>;
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
pub enum TabViewMessage {
    /// No operation.
    Noop,
    /// The selection has been changed by user.
    ChangeInputSelection,
    /// The selection prop has been changed.
    ChangePropSelection,
    /// Set the selection.
    SetSelection(Option<usize>),
    /// Set the rect.
    SetRect(Rect),
    /// Set the enabled state.
    SetEnabled(bool),
    /// Set the visible state.
    SetVisible(bool),
}

impl Component for TabView {
    type Error = Error;
    type Event = TabViewEvent;
    type Init<'a> = BorrowedContainer<'a>;
    type Message = TabViewMessage;

    async fn init(init: Self::Init<'_>, _sender: &ComponentSender<Self>) -> Result<Self> {
        let widget = sys::TabView::new(init)?;
        let Ok(selection_prop) = Child::<Prop<Option<usize>>>::init(None).await;
        Ok(Self {
            widget,
            selection_prop,
        })
    }

    async fn start(&mut self, sender: &ComponentSender<Self>) -> ! {
        let fut_select = async {
            loop {
                self.widget.wait_select().await;
                sender.post(TabViewMessage::ChangeInputSelection);
            }
        };
        let fut_props = async {
            start! {
                sender, default: TabViewMessage::Noop,
                self.selection_prop => { PropSinkEvent::Changed => TabViewMessage::ChangePropSelection },
            }
        };
        futures_util::future::join(fut_select, fut_props).await.0
    }

    async fn update_children(&mut self) -> Result<bool> {
        let Ok(r0) = self.selection_prop.update().await;
        Ok(r0)
    }

    async fn update(
        &mut self,
        message: Self::Message,
        sender: &ComponentSender<Self>,
    ) -> Result<bool> {
        match message {
            TabViewMessage::Noop => Ok(false),
            TabViewMessage::ChangeInputSelection => {
                let selection = self.widget.selection()?;
                self.selection_prop.post(PropSinkMessage::Set(selection));
                sender.output(TabViewEvent::Select);
                Ok(false)
            }
            TabViewMessage::ChangePropSelection => {
                let current = self.widget.selection()?;
                let prop_val = self.selection_prop.get();
                if &current != prop_val
                    && let Some(i) = prop_val
                {
                    self.widget.set_selection(*i)?;
                }
                Ok(true)
            }
            TabViewMessage::SetSelection(selection) => {
                self.selection_prop.set(selection);
                Ok(true)
            }
            TabViewMessage::SetRect(rect) => {
                self.set_rect(rect)?;
                Ok(true)
            }
            TabViewMessage::SetEnabled(enabled) => {
                self.set_enabled(enabled)?;
                Ok(false)
            }
            TabViewMessage::SetVisible(visible) => {
                self.set_visible(visible)?;
                Ok(true)
            }
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
pub enum TabViewItemMessage {
    /// No operation.
    Noop,
    /// Set the text.
    SetText(String),
}

impl Component for TabViewItem {
    type Error = Error;
    type Event = TabViewItemEvent;
    type Init<'a> = ();
    type Message = TabViewItemMessage;

    async fn init(_init: Self::Init<'_>, _sender: &ComponentSender<Self>) -> Result<Self> {
        let widget = sys::TabViewItem::new()?;
        Ok(Self { widget })
    }

    async fn update(
        &mut self,
        message: Self::Message,
        _sender: &ComponentSender<Self>,
    ) -> Result<bool> {
        match message {
            TabViewItemMessage::Noop => Ok(false),
            TabViewItemMessage::SetText(text) => {
                self.set_text(text)?;
                Ok(true)
            }
        }
    }
}

winio_handle::impl_as_container!(TabViewItem, widget);
