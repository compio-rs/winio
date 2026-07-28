use inherit_methods_macro::inherit_methods;
use winio_elm::{Child, Component, ComponentSender, PropSink, PropSinkEvent, start};
use winio_handle::BorrowedContainer;
use winio_primitive::{Enable, Failable, Layoutable, Point, Size, Visible};

use crate::{
    sys,
    sys::{Error, Result},
};

/// A scroll view that can contain other widgets and provide scrolling.
/// functionality.
#[derive(Debug)]
pub struct ScrollView {
    widget: sys::ScrollView,
    hscroll_prop: Child<PropSink<bool>>,
    vscroll_prop: Child<PropSink<bool>>,
    visible_prop: Child<PropSink<bool>>,
    enabled_prop: Child<PropSink<bool>>,
}

impl Failable for ScrollView {
    type Error = Error;
}

#[inherit_methods(from = "self.widget")]
impl ScrollView {
    /// Get if the horizontal scroll bar is visible.
    pub fn hscroll(&self) -> Result<bool>;

    /// Set if the horizontal scroll bar is visible.
    pub fn set_hscroll(&mut self, v: bool) -> Result<()>;

    /// Get if the vertical scroll bar is visible.
    pub fn vscroll(&self) -> Result<bool>;

    /// Set if the vertical scroll bar is visible.
    pub fn set_vscroll(&mut self, v: bool) -> Result<()>;

    /// Property for [`ScrollView::hscroll`].
    pub fn hscroll_prop(&self) -> &PropSink<bool> {
        &self.hscroll_prop
    }

    /// Property for [`ScrollView::vscroll`].
    pub fn vscroll_prop(&self) -> &PropSink<bool> {
        &self.vscroll_prop
    }

    /// Property for [`Visible::set_visible`].
    pub fn visible_prop(&self) -> &PropSink<bool> {
        &self.visible_prop
    }

    /// Property for [`Enable::set_enabled`].
    pub fn enabled_prop(&self) -> &PropSink<bool> {
        &self.enabled_prop
    }
}

#[inherit_methods(from = "self.widget")]
impl Visible for ScrollView {
    fn is_visible(&self) -> Result<bool>;

    fn set_visible(&mut self, v: bool) -> Result<()>;
}

#[inherit_methods(from = "self.widget")]
impl Enable for ScrollView {
    fn is_enabled(&self) -> Result<bool>;

    fn set_enabled(&mut self, v: bool) -> Result<()>;
}

#[inherit_methods(from = "self.widget")]
impl Layoutable for ScrollView {
    fn loc(&self) -> Result<Point>;

    fn set_loc(&mut self, p: Point) -> Result<()>;

    fn size(&self) -> Result<Size>;

    fn set_size(&mut self, v: Size) -> Result<()>;
}

/// Events of [`ScrollView`].
#[derive(Debug)]
#[non_exhaustive]
pub enum ScrollViewEvent {}

/// Messages of [`ScrollView`].
#[derive(Debug)]
#[non_exhaustive]
pub enum ScrollViewMessage {
    /// No operation.
    Noop,
    /// The horizontal scroll visibility has been changed.
    ChangeHscroll,
    /// The vertical scroll visibility has been changed.
    ChangeVscroll,
    /// The visible state has been changed.
    ChangeVisible,
    /// The enabled state has been changed.
    ChangeEnabled,
}

impl Component for ScrollView {
    type Error = Error;
    type Event = ScrollViewEvent;
    type Init<'a> = BorrowedContainer<'a>;
    type Message = ScrollViewMessage;

    async fn init(init: Self::Init<'_>, _sender: &ComponentSender<Self>) -> Result<Self> {
        let widget = sys::ScrollView::new(init)?;
        let Ok(hscroll_prop) = Child::<PropSink<bool>>::init(widget.hscroll()?).await;
        let Ok(vscroll_prop) = Child::<PropSink<bool>>::init(widget.vscroll()?).await;
        let Ok(visible_prop) = Child::<PropSink<bool>>::init(true).await;
        let Ok(enabled_prop) = Child::<PropSink<bool>>::init(true).await;
        Ok(Self {
            widget,
            hscroll_prop,
            vscroll_prop,
            visible_prop,
            enabled_prop,
        })
    }

    async fn start(&mut self, sender: &ComponentSender<Self>) -> ! {
        let fut_native = self.widget.start();
        let fut_props = async {
            start! {
                sender, default: ScrollViewMessage::Noop,
                self.hscroll_prop => { PropSinkEvent::Changed => ScrollViewMessage::ChangeHscroll },
                self.vscroll_prop => { PropSinkEvent::Changed => ScrollViewMessage::ChangeVscroll },
                self.visible_prop => { PropSinkEvent::Changed => ScrollViewMessage::ChangeVisible },
                self.enabled_prop => { PropSinkEvent::Changed => ScrollViewMessage::ChangeEnabled },
            }
        };
        futures_util::future::join(fut_native, fut_props).await.0
    }

    async fn update_children(&mut self) -> Result<bool> {
        let Ok(r0) = self.hscroll_prop.update().await;
        let Ok(r1) = self.vscroll_prop.update().await;
        let Ok(r2) = self.visible_prop.update().await;
        let Ok(r3) = self.enabled_prop.update().await;
        Ok(r0 || r1 || r2 || r3)
    }

    async fn update(
        &mut self,
        message: Self::Message,
        _sender: &ComponentSender<Self>,
    ) -> Result<bool> {
        match message {
            ScrollViewMessage::Noop => Ok(false),
            ScrollViewMessage::ChangeHscroll => {
                self.widget.set_hscroll(**self.hscroll_prop)?;
                Ok(true)
            }
            ScrollViewMessage::ChangeVscroll => {
                self.widget.set_vscroll(**self.vscroll_prop)?;
                Ok(true)
            }
            ScrollViewMessage::ChangeVisible => {
                self.widget.set_visible(**self.visible_prop)?;
                Ok(true)
            }
            ScrollViewMessage::ChangeEnabled => {
                self.widget.set_enabled(**self.enabled_prop)?;
                Ok(true)
            }
        }
    }
}

winio_handle::impl_as_widget!(ScrollView, widget);
winio_handle::impl_as_container!(ScrollView, widget);
